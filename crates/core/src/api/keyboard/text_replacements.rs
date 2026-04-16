use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use color_eyre::Result;
use derive_where::derive_where;
use enigo::{Direction, Key, Keyboard};
use indexmap::IndexMap;
use itertools::Itertools;
use macros::options;
use parking_lot::Mutex;
use rquickjs::{AsyncContext, Coerced};
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    api::{
        image::{Image, js::JsImage},
        js::event_handle::{HandleId, HandleRegistry},
        macros::{MacroData, PlayConfig, js::JsMacro, player::MacroPlayer},
    },
    runtime::{
        Runtime, WithUserData,
        events::{KeyboardKeyEvent, KeyboardTextEvent},
    },
    scripting::callbacks::FunctionKey,
};

#[derive(Clone)]
#[derive_where(Debug)]
pub enum Replacement {
    Text(String),
    Image(Image),
    Macro(Arc<MacroData>),
    JsCallback(#[derive_where(skip)] (AsyncContext, FunctionKey)),
}

/// Options for `onText`.
#[options]
#[derive(Clone, Copy, Debug)]
pub struct OnTextOptions {
    /// Erase the typed text before inserting the replacement.
    /// Set to `false` to trigger an action without replacing the typed text.
    #[default(true)]
    pub erase: bool,

    /// When replacing with text, use the clipboard (Ctrl+V) instead of simulated keystrokes.
    /// Replacing with an image always uses the clipboard.
    pub use_clipboard_for_text: bool,

    /// Save and restore the clipboard contents around a clipboard-based replacement.
    #[default(true)]
    pub save_restore_clipboard: bool,
}

/// One registered handler for a given text trigger.
struct TextHandler {
    id: HandleId,
    replacement: Replacement,
    options: OnTextOptions,
}

impl std::fmt::Debug for TextHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextHandler")
            .field("id", &self.id)
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

/// Registry: sorted longest-first by trigger string, each entry has ≥1 handlers.
type Registry = IndexMap<String, Vec<TextHandler>>;

struct PendingTextReplacement {
    key: Key,
    scan_code: u32,
    actions: Vec<PendingTextReplacementAction>,
}

struct PendingTextReplacementAction {
    trigger: String,
    replacement_data: ReplacementData,
    options: OnTextOptions,
}

#[derive(Clone, Debug)]
pub struct TextReplacements {
    registry: Arc<Mutex<Registry>>,
    max_graphemes: Arc<AtomicUsize>,
    runtime: Arc<Runtime>,
    macro_player: Arc<MacroPlayer>,
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
    listener_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl TextReplacements {
    pub fn new(
        runtime: Arc<Runtime>,
        macro_player: Arc<MacroPlayer>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            registry: Arc::new(Mutex::new(IndexMap::default())),
            max_graphemes: Arc::new(AtomicUsize::new(0)),
            runtime,
            macro_player,
            task_tracker,
            cancellation_token,
            listener_cancellation_token: Arc::new(Mutex::new(None)),
        }
    }

    async fn on_key(
        event: KeyboardKeyEvent,
        buffer: &mut StringRingBuffer,
        runtime: Arc<Runtime>,
        macro_player: Arc<MacroPlayer>,
        pending_replacement: &mut Option<PendingTextReplacement>,
    ) -> Result<()> {
        if event.is_injected {
            return Ok(());
        }

        if pending_replacement
            .as_ref()
            .is_some_and(|pending| Self::is_pending_trigger_release(&event, pending))
        {
            let pending = pending_replacement
                .take()
                .expect("pending replacement should exist when matching release is detected");
            Self::apply_pending_replacement(pending, runtime, macro_player)?;
            buffer.clear();
            return Ok(());
        }

        if pending_replacement.is_some() || event.direction.is_release() {
            return Ok(());
        }

        match event.key {
            Key::Backspace => buffer.pop(),
            Key::Escape => buffer.clear(),
            Key::LeftArrow | Key::RightArrow | Key::UpArrow | Key::DownArrow => buffer.clear(),
            _ => {}
        }
        Ok(())
    }

    async fn on_text(
        event: KeyboardTextEvent,
        buffer: &mut StringRingBuffer,
        max_graphemes: Arc<AtomicUsize>,
        registry: Arc<Mutex<Registry>>,
        pending_replacement: &mut Option<PendingTextReplacement>,
    ) -> Result<()> {
        if event.is_injected || pending_replacement.is_some() {
            return Ok(());
        }

        let max_graphemes_val = max_graphemes.load(Ordering::Relaxed);
        if max_graphemes_val == 0 {
            buffer.clear();
            return Ok(());
        }

        buffer.add_str_and_set_max_graphemes(&event.text, max_graphemes_val);

        // Collect all handlers for the longest matching trigger.
        // The registry is sorted longest-first, so `.find()` gives that trigger directly.
        let matches: Vec<(String, Replacement, OnTextOptions)> = {
            let reg = registry.lock();
            let Some((trigger, handlers)) = reg
                .iter()
                .find(|(trigger, _)| buffer.value().ends_with(trigger.as_str()))
            else {
                return Ok(());
            };

            handlers
                .iter()
                .map(|handler| {
                    (
                        trigger.clone(),
                        handler.replacement.clone(),
                        handler.options,
                    )
                })
                .collect()
        };

        let mut actions = Vec::with_capacity(matches.len());
        for (trigger, replacement, options) in matches {
            let replacement_data = match replacement {
                Replacement::Text(text) => ReplacementData::Text(text),
                Replacement::Image(image) => ReplacementData::Image(image),
                Replacement::Macro(data) => ReplacementData::Macro(data),
                Replacement::JsCallback((context, function_key)) => {
                    let trigger_for_callback = trigger.clone();

                    // Phase 1: queue the call inside a non-yielding async_with! so the
                    // rquickjs scheduler's waker is not overwritten by this task's waker.
                    let prepare_result = context
                        .async_with(async |ctx| {
                            ctx.user_data()
                                .callbacks()
                                .prepare_call(&ctx, function_key, Vec::new())
                        })
                        .await;

                    let Some((call_id, finished_receiver)) = prepare_result else {
                        warn!(
                            trigger = %trigger_for_callback,
                            ?function_key,
                            "onText callback worker is not running; keeping typed text"
                        );
                        continue;
                    };

                    // Phase 2: wait for completion outside any async_with! so the
                    // scheduler's waker is undisturbed and waitForKeys can still resolve.
                    if finished_receiver.await.is_err() {
                        warn!(
                            trigger = %trigger_for_callback,
                            ?function_key,
                            "onText callback worker dropped before finishing; keeping typed text"
                        );
                        continue;
                    }

                    // Phase 3: retrieve and process the result.
                    let callback_outcome = context.async_with(async |ctx| {
                        let value = match ctx
                            .user_data()
                            .callbacks()
                            .retrieve_result(&ctx, call_id)
                        {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(
                                    trigger = %trigger_for_callback,
                                    ?function_key,
                                    error = %error,
                                    "onText callback failed; keeping typed text"
                                );
                                return CallbackOutcome::KeepTypedText;
                            }
                        };

                        if value.is_undefined() || value.is_null() {
                            // void return — callback ran, nothing to insert.
                            return CallbackOutcome::Apply(ReplacementData::None);
                        }

                        if let Ok(image) = value.get::<JsImage>() {
                            return CallbackOutcome::Apply(ReplacementData::Image(image.into_inner()));
                        }

                        if let Ok(r#macro) = value.get::<JsMacro>() {
                            return CallbackOutcome::Macro(r#macro.data());
                        }

                        match value.get::<Coerced<String>>() {
                            Ok(text) => CallbackOutcome::Apply(ReplacementData::Text(text.0)),
                            Err(error) => {
                                warn!(
                                    trigger = %trigger_for_callback,
                                    ?function_key,
                                    error = %error,
                                    "onText callback did not return image, macro, string, or void; keeping typed text"
                                );
                                CallbackOutcome::KeepTypedText
                            }
                        }
                    }).await;

                    match callback_outcome {
                        CallbackOutcome::Apply(replacement_data) => replacement_data,
                        CallbackOutcome::Macro(data) => ReplacementData::Macro(data),
                        CallbackOutcome::KeepTypedText => continue,
                    }
                }
            };
            actions.push(PendingTextReplacementAction {
                trigger,
                replacement_data,
                options,
            });
        }

        if !actions.is_empty() {
            *pending_replacement = Some(PendingTextReplacement {
                key: event.key,
                scan_code: event.scan_code,
                actions,
            });
        }
        Ok(())
    }

    fn is_pending_trigger_release(
        event: &KeyboardKeyEvent,
        pending: &PendingTextReplacement,
    ) -> bool {
        event.direction.is_release()
            && event.scan_code == pending.scan_code
            && event.key == pending.key
    }

    fn apply_pending_replacement(
        pending: PendingTextReplacement,
        runtime: Arc<Runtime>,
        macro_player: Arc<MacroPlayer>,
    ) -> Result<()> {
        for action in pending.actions {
            let (backspaces, replacement_data) = replacement_plan(
                &action.trigger,
                action.replacement_data,
                action.options.erase,
            );

            if backspaces > 0 {
                let enigo = runtime.enigo();
                let mut enigo = enigo.lock();
                for _ in 0..backspaces {
                    enigo.key(Key::Backspace, Direction::Click)?;
                }
            }

            match replacement_data {
                ReplacementData::None => {}
                ReplacementData::Macro(data) => {
                    macro_player.play_detached(data, PlayConfig::default());
                }
                ReplacementData::Text(text) => {
                    if action.options.use_clipboard_for_text {
                        let clipboard = runtime.clipboard();
                        let saved = if action.options.save_restore_clipboard {
                            match clipboard.save(None) {
                                Ok(data) => Some(data),
                                Err(error) => {
                                    warn!(
                                        trigger = %action.trigger,
                                        error = %error,
                                        "failed to save clipboard before text replacement"
                                    );
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        clipboard.set_text(&text, None)?;
                        let enigo = runtime.enigo();
                        let mut enigo = enigo.lock();
                        enigo.key(Key::Control, Direction::Press)?;
                        enigo.key(Key::Unicode('v'), Direction::Press)?;
                        enigo.key(Key::Unicode('v'), Direction::Release)?;
                        enigo.key(Key::Control, Direction::Release)?;
                        if let Some(data) = saved
                            && let Err(error) = clipboard.restore(data, None)
                        {
                            warn!(
                                trigger = %action.trigger,
                                error = %error,
                                "failed to restore clipboard after text replacement"
                            );
                        }
                    } else {
                        let enigo = runtime.enigo();
                        let mut enigo = enigo.lock();
                        enigo.text(&text)?;
                    }
                }
                ReplacementData::Image(image) => {
                    let clipboard = runtime.clipboard();
                    let saved = if action.options.save_restore_clipboard {
                        Some(clipboard.save(None)?)
                    } else {
                        None
                    };
                    clipboard.set_image(image, None)?;
                    let enigo = runtime.enigo();
                    let mut enigo = enigo.lock();
                    enigo.key(Key::Control, Direction::Press)?;
                    enigo.key(Key::Unicode('v'), Direction::Press)?;
                    enigo.key(Key::Unicode('v'), Direction::Release)?;
                    enigo.key(Key::Control, Direction::Release)?;
                    if let Some(data) = saved {
                        clipboard.restore(data, None)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn ensure_listener_started(&self) {
        let mut listener_cancellation_token = self.listener_cancellation_token.lock();
        if listener_cancellation_token.is_some() {
            return;
        }

        let worker_cancellation_token = self.cancellation_token.child_token();
        *listener_cancellation_token = Some(worker_cancellation_token.clone());
        drop(listener_cancellation_token);

        let local_runtime = self.runtime.clone();
        let local_registry = self.registry.clone();
        let local_max_graphemes = self.max_graphemes.clone();
        let local_macro_player = self.macro_player.clone();
        self.task_tracker.spawn(async move {
            let text_guard = local_runtime.keyboard_text();
            let mut text_receiver = text_guard.subscribe();
            let keys_guard = local_runtime.keyboard_keys();
            let mut keys_receiver = keys_guard.subscribe();

            let mut buffer = StringRingBuffer::default();
            let mut pending_replacement = None;

            loop {
                select! {
                    _ = worker_cancellation_token.cancelled() => { break; }
                    text = text_receiver.recv() => {
                        let Ok(text) = text else { break; };
                        Self::on_text(
                            text,
                            &mut buffer,
                            local_max_graphemes.clone(),
                            local_registry.clone(),
                            &mut pending_replacement,
                        ).await?;
                    },
                    key = keys_receiver.recv() => {
                        let Ok(key) = key else { break; };
                        Self::on_key(
                            key,
                            &mut buffer,
                            local_runtime.clone(),
                            local_macro_player.clone(),
                            &mut pending_replacement,
                        ).await?;
                    },
                }
            }

            Result::<()>::Ok(())
        });
    }

    fn stop_listener_if_running(&self) {
        let cancellation_token = self.listener_cancellation_token.lock().take();
        if let Some(cancellation_token) = cancellation_token {
            cancellation_token.cancel();
        }
    }

    pub fn add(
        &self,
        id: HandleId,
        trigger: &str,
        replacement: Replacement,
        options: OnTextOptions,
    ) {
        let was_empty = {
            let mut reg = self.registry.lock();
            let was_empty = reg.is_empty();

            let is_new_trigger = !reg.contains_key(trigger);
            reg.entry(trigger.to_string())
                .or_default()
                .push(TextHandler {
                    id,
                    replacement,
                    options,
                });

            // Keep the IndexMap sorted longest-first only when a new trigger key was added.
            if is_new_trigger {
                reg.sort_by(|a, _, b, _| b.graphemes(true).count().cmp(&a.graphemes(true).count()));
            }

            let max = reg
                .keys()
                .map(|key| key.graphemes(true).count())
                .max()
                .unwrap_or(0);
            self.max_graphemes.store(max, Ordering::Relaxed);

            was_empty
        };

        if was_empty {
            self.ensure_listener_started();
            self.runtime.increase_background_tasks_counter();
        }
    }

    pub fn remove(&self, id: HandleId) {
        let became_empty = {
            let mut reg = self.registry.lock();
            let was_empty = reg.is_empty();

            reg.retain(|_, handlers| {
                handlers.retain(|h| h.id != id);
                !handlers.is_empty()
            });

            let max = reg
                .keys()
                .map(|key| key.graphemes(true).count())
                .max()
                .unwrap_or(0);
            self.max_graphemes.store(max, Ordering::Relaxed);

            !was_empty && reg.is_empty()
        };

        if became_empty {
            self.stop_listener_if_running();
            self.runtime.decrease_background_tasks_counter();
        }
    }

    pub fn clear(&self) {
        {
            let mut reg = self.registry.lock();
            if reg.is_empty() {
                return;
            }
            reg.clear();
            self.max_graphemes.store(0, Ordering::Relaxed);
        }

        self.stop_listener_if_running();
        self.runtime.decrease_background_tasks_counter();
    }
}

impl HandleRegistry for TextReplacements {
    fn remove_handle(&self, id: HandleId) {
        self.remove(id);
    }
}

fn grapheme_prefix_len(left: &str, right: &str) -> usize {
    left.graphemes(true)
        .zip(right.graphemes(true))
        .take_while(|(a, b)| a == b)
        .count()
}

enum ReplacementData {
    Text(String),
    Image(Image),
    Macro(Arc<MacroData>),
    None,
}

enum CallbackOutcome {
    Apply(ReplacementData),
    Macro(Arc<MacroData>),
    KeepTypedText,
}

fn replacement_plan(
    trigger: &str,
    replacement_data: ReplacementData,
    erase: bool,
) -> (usize, ReplacementData) {
    if !erase {
        return (0, replacement_data);
    }

    let trigger_grapheme_count = trigger.graphemes(true).count();
    match replacement_data {
        ReplacementData::Text(text) => {
            let common_prefix_len = grapheme_prefix_len(trigger, &text);
            let backspaces = trigger_grapheme_count.saturating_sub(common_prefix_len);
            let suffix = text.graphemes(true).skip(common_prefix_len).collect();
            (backspaces, ReplacementData::Text(suffix))
        }
        ReplacementData::Image(image) => (trigger_grapheme_count, ReplacementData::Image(image)),
        ReplacementData::Macro(data) => (trigger_grapheme_count, ReplacementData::Macro(data)),
        ReplacementData::None => (trigger_grapheme_count, ReplacementData::None),
    }
}

#[derive(Debug, Default)]
struct StringRingBuffer {
    buffer: String,
    max_graphemes: usize,
}

impl StringRingBuffer {
    #[cfg(test)]
    fn new(max_graphemes: usize) -> Self {
        Self {
            buffer: String::default(),
            max_graphemes,
        }
    }

    #[cfg(test)]
    fn add_char(&mut self, ch: char) {
        self.buffer.push(ch);
        self.update();
    }

    pub fn add_str_and_set_max_graphemes(&mut self, text: &str, max_graphemes: usize) {
        self.buffer.push_str(text);
        self.max_graphemes = max_graphemes;
        self.update();
    }

    #[cfg(test)]
    fn add_str(&mut self, str: &str) {
        self.buffer.push_str(str);
        self.update();
    }

    #[cfg(test)]
    fn set_max_grapheme_count(&mut self, max_graphemes: usize) {
        self.max_graphemes = max_graphemes;
        self.update();
    }

    pub fn pop(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        let (start, _) = match self.buffer.grapheme_indices(true).next_back() {
            Some(pair) => pair,
            None => return,
        };
        self.buffer.truncate(start);
    }

    pub fn value(&self) -> &str {
        &self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    fn update(&mut self) {
        if self.max_graphemes == 0 {
            self.buffer.clear();
            return;
        }
        if self.buffer.graphemes(true).count() <= self.max_graphemes {
            return;
        }
        let mut graphemes = self.buffer.graphemes(true).collect_vec();
        graphemes.drain(0..(graphemes.len() - self.max_graphemes));
        self.buffer = graphemes.concat();
    }
}

#[cfg(test)]
mod tests {
    use unicode_segmentation::UnicodeSegmentation;

    use super::*;

    #[test]
    fn test_grapheme_prefix_len() {
        assert_eq!(grapheme_prefix_len("", "abc"), 0);
        assert_eq!(grapheme_prefix_len("a", "abc"), 1);
        assert_eq!(grapheme_prefix_len("abcd", "abc"), 3);
        assert_eq!(grapheme_prefix_len("", ""), 0);
    }

    #[test]
    fn ascii_basic() {
        let mut ring_buffer = StringRingBuffer::new(3);
        ring_buffer.add_str("abc");
        assert_eq!(ring_buffer.value(), "abc");
        ring_buffer.add_char('d');
        assert_eq!(ring_buffer.value(), "bcd");
        ring_buffer.add_str("ef");
        assert_eq!(ring_buffer.value(), "def");
    }

    #[test]
    fn combining_mark_counts_as_one_grapheme() {
        let mut ring_buffer = StringRingBuffer::new(1);
        let composed = "e\u{0302}";
        assert_eq!(composed.graphemes(true).count(), 1);
        ring_buffer.add_str(composed);
        assert_eq!(ring_buffer.value().graphemes(true).count(), 1);
        assert_eq!(ring_buffer.value(), composed);
    }

    #[test]
    fn emoji_with_skin_tone_is_one_grapheme() {
        let mut ring_buffer = StringRingBuffer::new(1);
        let grapheme = "👍🏽";
        assert_eq!(grapheme.graphemes(true).count(), 1);
        ring_buffer.add_str(grapheme);
        assert_eq!(ring_buffer.value(), grapheme);
        ring_buffer.add_str("A");
        assert_eq!(ring_buffer.value(), "A");
    }

    #[test]
    fn zwj_sequence_is_one_grapheme() {
        let mut ring_buffer = StringRingBuffer::new(2);
        let emoji = "👨‍👩‍👧‍👦";
        assert_eq!(emoji.graphemes(true).count(), 1);
        ring_buffer.add_str(emoji);
        ring_buffer.add_str("X");
        assert_eq!(ring_buffer.value(), format!("{emoji}X"));
    }

    #[test]
    fn flag_is_one_grapheme_two_scalars() {
        let mut ring_buffer = StringRingBuffer::new(2);
        let flag = "🇬🇧";
        assert_eq!(flag.chars().count(), 2);
        assert_eq!(flag.graphemes(true).count(), 1);
        ring_buffer.add_str("A");
        ring_buffer.add_str(flag);
        assert_eq!(ring_buffer.value(), format!("A{flag}"));
        ring_buffer.add_str("B");
        assert_eq!(ring_buffer.value(), format!("{flag}B"));
    }

    #[test]
    fn shrinking_and_expanding_max() {
        let mut ring_buffer = StringRingBuffer::new(5);
        ring_buffer.add_str("abcde");
        assert_eq!(ring_buffer.value(), "abcde");
        ring_buffer.set_max_grapheme_count(2);
        assert_eq!(ring_buffer.value(), "de");
        ring_buffer.set_max_grapheme_count(4);
        ring_buffer.add_str("FG");
        assert_eq!(ring_buffer.value(), "deFG");
    }

    #[test]
    fn zero_max_results_empty() {
        let mut ring_buffer = StringRingBuffer::new(0);
        ring_buffer.add_str("whatever 👍🏽");
        assert_eq!(ring_buffer.value(), "");
        ring_buffer.set_max_grapheme_count(1);
        ring_buffer.add_str("X");
        assert_eq!(ring_buffer.value(), "X");
    }

    #[test]
    fn pending_replacement_matches_exact_release() {
        let pending = PendingTextReplacement {
            key: Key::Unicode('a'),
            scan_code: 42,
            actions: Vec::new(),
        };

        let matching_release = KeyboardKeyEvent::new(
            Key::Unicode('a'),
            42,
            crate::types::input::Direction::Release,
            false,
            String::new(),
            false,
        );
        assert!(TextReplacements::is_pending_trigger_release(
            &matching_release,
            &pending,
        ));

        let wrong_scan_code = KeyboardKeyEvent::new(
            Key::Unicode('a'),
            99,
            crate::types::input::Direction::Release,
            false,
            String::new(),
            false,
        );
        assert!(!TextReplacements::is_pending_trigger_release(
            &wrong_scan_code,
            &pending,
        ));

        let wrong_direction = KeyboardKeyEvent::new(
            Key::Unicode('a'),
            42,
            crate::types::input::Direction::Press,
            false,
            String::new(),
            false,
        );
        assert!(!TextReplacements::is_pending_trigger_release(
            &wrong_direction,
            &pending,
        ));
    }
}
