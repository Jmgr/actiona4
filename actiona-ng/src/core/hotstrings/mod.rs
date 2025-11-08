use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use enigo::{Direction, Key, Keyboard};
use eyre::Result;
use indexmap::IndexMap;
use itertools::Itertools;
use parking_lot::Mutex;
use tokio::{select, time::sleep};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use unicode_segmentation::UnicodeSegmentation;

use crate::runtime::{
    Runtime,
    events::{KeyboardKeyEvent, KeyboardTextEvent},
};

#[derive(Clone, Debug)]
pub struct Hotstrings {
    hotstrings: Arc<Mutex<IndexMap<String, String>>>,
    max_graphemes: Arc<AtomicUsize>,
}

impl Hotstrings {
    pub fn new(
        runtime: Arc<Runtime>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let hotstrings = Arc::new(Mutex::new(IndexMap::<String, String>::default()));
        let max_graphemes = Arc::new(AtomicUsize::new(0));

        let local_hotstrings = hotstrings.clone();
        let local_max_graphemes = max_graphemes.clone();

        task_tracker.spawn(async move {
            let text_guard = runtime.keyboard_text();
            let mut text_receiver = text_guard.subscribe();
            let keys_guard = runtime.keyboard_keys();
            let mut keys_receiver = keys_guard.subscribe();

            let mut buffer = StringRingBuffer::default();

            let trigger_characters = HashSet::from(['\n', '\r', ',', '.', ' ']); // TODO: add parameter

            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    text = text_receiver.recv() => {
                        let Ok(text) = text else {
                            break;
                        };

                        Self::on_text(text, &mut buffer, &trigger_characters, local_max_graphemes.clone(), local_hotstrings.clone(), runtime.clone()).await?;
                    },
                    key = keys_receiver.recv() => {
                        let Ok(key) = key else {
                            break;
                        };

                        Self::on_key(key, &mut buffer).await?;
                    },
                }
            }

            Result::<()>::Ok(())
        });

        Self {
            hotstrings,
            max_graphemes,
        }
    }

    async fn on_key(event: KeyboardKeyEvent, buffer: &mut StringRingBuffer) -> Result<()> {
        if event.is_injected || event.direction.is_release() {
            return Ok(());
        }

        match event.key {
            Key::Backspace => buffer.pop(),
            Key::LeftArrow | Key::RightArrow | Key::UpArrow | Key::DownArrow => buffer.clear(),
            _ => {}
        }

        Ok(())
    }

    async fn on_text(
        event: KeyboardTextEvent,
        buffer: &mut StringRingBuffer,
        trigger_characters: &HashSet<char>,
        max_graphemes: Arc<AtomicUsize>,
        hotstrings: Arc<Mutex<IndexMap<String, String>>>,
        runtime: Arc<Runtime>,
    ) -> Result<()> {
        if event.is_injected {
            return Ok(());
        }

        // No hotstrings
        let max_graphemes = max_graphemes.load(Ordering::Relaxed);

        if max_graphemes == 0 {
            buffer.clear();
            return Ok(());
        }

        /*
        if !trigger_characters.contains(&event.character) {
            // Not a trigger character
            buffer.add_char_and_set_max_graphemes(event.character, max_graphemes);
            return Ok(());
        }
        */

        let (backspaces, suffix) = {
            let hotstrings = hotstrings.lock();

            let key_char = event.character;
            buffer.add_char_and_set_max_graphemes(key_char, max_graphemes);

            // Look for the longest match
            let hotstring = hotstrings
                .iter()
                .find(|(key, _)| buffer.value().ends_with(*key));

            let Some((key, replacement)) = hotstring else {
                // No match
                return Ok(());
            };

            let grapheme_prefix_len = grapheme_prefix_len(key, replacement);
            let backspaces = key.graphemes(true).count() - grapheme_prefix_len; // + 1; // We add 1 for the trigger char
            let replacement = replacement.graphemes(true).collect_vec();
            let mut suffix = replacement[grapheme_prefix_len..].concat();

            //suffix.push(key_char); // Add the trigger character back

            (backspaces, suffix)
        };

        {
            let enigo = runtime.enigo();
            let mut guard = enigo.lock();

            for _ in 0..backspaces {
                guard.key(Key::Backspace, Direction::Click)?;
            }

            // Write the replacement
            guard.text(&suffix)?;
        };

        // Clear the buffer to prevent firing again
        buffer.clear();

        Ok(())
    }

    pub fn add(&self, key: &str, replacement: &str) {
        let mut hotstrings = self.hotstrings.lock();

        hotstrings.insert_sorted_by_key(key.to_string(), replacement.to_string(), |a, b| {
            b.graphemes(true).count().cmp(&a.graphemes(true).count())
        });

        let max_graphemes = hotstrings
            .keys()
            .map(|key| key.graphemes(true).count())
            .max()
            .expect("hotstrings should contain at least one entry");

        self.max_graphemes.store(max_graphemes, Ordering::Relaxed);
    }

    pub fn remove(&self, key: &str) {
        let mut hotstrings = self.hotstrings.lock();

        hotstrings.shift_remove(key);

        self.max_graphemes.store(0, Ordering::Relaxed);
    }
}

fn grapheme_prefix_len(left: &str, right: &str) -> usize {
    left.graphemes(true)
        .zip(right.graphemes(true))
        .take_while(|(a, b)| a == b)
        .count()
}

#[derive(Debug, Default)]
struct StringRingBuffer {
    buffer: String,
    max_graphemes: usize,
}

impl StringRingBuffer {
    pub fn new(max_graphemes: usize) -> Self {
        Self {
            buffer: String::default(),
            max_graphemes,
        }
    }

    pub fn add_char(&mut self, ch: char) {
        self.buffer.push(ch);
        self.update();
    }

    pub fn add_char_and_set_max_graphemes(&mut self, ch: char, max_graphemes: usize) {
        self.buffer.push(ch);
        self.max_graphemes = max_graphemes;
        self.update();
    }

    pub fn add_str(&mut self, str: &str) {
        self.buffer.push_str(str);
        self.update();
    }

    pub fn set_max_grapheme_count(&mut self, max_graphemes: usize) {
        self.max_graphemes = max_graphemes;
        self.update();
    }

    pub fn pop(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        // Find the start byte index and the grapheme slice of the last cluster.
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
    use std::time::Duration;

    use tokio::time::sleep;
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
        // 'e' + COMBINING CIRCUMFLEX ACCENT -> "ê" as one grapheme
        let mut ring_buffer = StringRingBuffer::new(1);

        let composed = "e\u{0302}";
        assert_eq!(composed.graphemes(true).count(), 1);

        ring_buffer.add_str(composed);
        assert_eq!(ring_buffer.value().graphemes(true).count(), 1);
        assert_eq!(ring_buffer.value(), composed);
    }

    #[test]
    fn emoji_with_skin_tone_is_one_grapheme() {
        // 👍🏽 is a single grapheme (base + modifier)
        let mut ring_buffer = StringRingBuffer::new(1);
        let grapheme = "👍🏽";
        assert_eq!(grapheme.graphemes(true).count(), 1);

        ring_buffer.add_str(grapheme);
        assert_eq!(ring_buffer.value(), grapheme);

        // Add another grapheme and ensure old one is dropped when max=1
        ring_buffer.add_str("A");
        assert_eq!(ring_buffer.value(), "A");
    }

    #[test]
    fn zwj_sequence_is_one_grapheme() {
        // Family emoji is a ZWJ sequence: still one grapheme
        let mut ring_buffer = StringRingBuffer::new(2);
        let emoji = "👨‍👩‍👧‍👦";
        assert_eq!(emoji.graphemes(true).count(), 1);

        ring_buffer.add_str(emoji);
        ring_buffer.add_str("X");
        // Keep last 2 graphemes: [family, X]
        assert_eq!(ring_buffer.value(), format!("{emoji}X"));
    }

    #[test]
    fn flag_is_one_grapheme_two_scalars() {
        // 🇬🇧 is two regional indicators but one grapheme
        let mut ring_buffer = StringRingBuffer::new(2);
        let flag = "🇬🇧";
        assert_eq!(flag.chars().count(), 2);
        assert_eq!(flag.graphemes(true).count(), 1);

        ring_buffer.add_str("A");
        ring_buffer.add_str(flag);
        assert_eq!(ring_buffer.value(), format!("A{flag}"));

        ring_buffer.add_str("B");
        // Should keep last 2 graphemes: [flag, B]
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
        // Expanding doesn't bring back truncated prefix; future adds still bounded
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
    #[ignore]
    fn test_replacement() {
        Runtime::test(async |runtime| {
            let task_tracker = TaskTracker::new();
            let cancellation_token = CancellationToken::new();

            let hotstrings = Hotstrings::new(runtime, task_tracker.clone(), cancellation_token);
            hotstrings.add(":)", "😀");
            hotstrings.add(":D", "😁");
            hotstrings.add("beaver", "🦫");

            sleep(Duration::from_secs(6000)).await;

            task_tracker.close();
            task_tracker.wait().await;
        });
    }
}
