//! @verbatim /**
//! @verbatim  * An action that fires when a trigger is activated: either a {@link Macro} to play
//! @verbatim  * directly, or a (possibly async) callback that optionally returns one.
//! @verbatim  *
//! @verbatim  * ```ts
//! @verbatim  * keyboard.onKey(Key.F5, myMacro);               // play macro directly
//! @verbatim  * keyboard.onKey(Key.F5, () => myMacro);         // callback returning macro
//! @verbatim  * keyboard.onKey(Key.F5, async () => { ... });   // async callback
//! @verbatim  * ```
//! @verbatim  */
//! @verbatim type TriggerAction = Macro | (() => Macro | void | Promise<Macro | void>);

use std::{collections::HashSet, sync::Arc, time::Duration};

use humantime::format_duration;
use macros::{FromJsObject, js_class, js_methods, options, platform};
use rquickjs::{
    Class, Ctx, Exception, JsLifetime, Object, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::sync::mpsc;
use tracing::instrument;

use super::{MacroData, PlayConfig, PlayProgress, RecordConfig, player::MacroPlayer, record_impl};
use crate::{
    IntoJsResult,
    api::{
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, registration_target},
            date::date_from_system_time,
            duration::JsDuration,
            task::{progress_task_with_token, task_with_token},
        },
        keyboard::js::{JsKey, JsStandardKey},
        mouse::Mouse,
    },
    runtime::{Runtime, WithUserData},
    types::display::display_with_type,
};

/// A recorded macro that can be replayed or saved to disk.
///
/// ```ts
/// // Record a macro
/// const m = await macros.record({ stopKeys: [Key.Escape] });
///
/// // Save and reload
/// await m.save("my_macro.amac");
/// const loaded = await Macro.load("my_macro.amac");
///
/// // Play back
/// await macros.play(loaded, { speed: 1.5 });
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsMacro {
    data: Arc<MacroData>,
}

impl<'js> HostClass<'js> for JsMacro {}

impl<'js> Trace<'js> for JsMacro {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsMacro {
    pub(crate) fn data(&self) -> Arc<MacroData> {
        self.data.clone()
    }
}

#[js_methods]
impl JsMacro {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(Exception::throw_message(
            &ctx,
            "Macro cannot be instantiated directly; use macros.record() or Macro.load() instead",
        ))
    }

    /// Saves this macro to a gzip-compressed JSON file.
    ///
    /// ```ts
    /// await macro.save("recording.amac");
    /// ```
    pub async fn save(&self, ctx: Ctx<'_>, path: String) -> Result<()> {
        self.data
            .save(path, ctx.user_data().task_tracker())
            .await
            .into_js_result(&ctx)
    }

    /// Loads a macro from a gzip-compressed JSON file previously written by `save()`.
    ///
    /// ```ts
    /// const loaded = await Macro.load("recording.amac");
    /// await macros.play(loaded);
    /// ```
    #[qjs(static)]
    pub async fn load(ctx: Ctx<'_>, path: String) -> Result<Self> {
        MacroData::load(path, ctx.user_data().task_tracker())
            .await
            .map(|data| Self {
                data: Arc::new(data),
            })
            .into_js_result(&ctx)
    }

    /// Returns the total number of events in this macro.
    #[must_use]
    #[get]
    pub fn event_count(&self) -> u64 {
        u64::try_from(self.data.events.len()).unwrap_or(u64::MAX)
    }

    /// Returns the total duration of the recording in seconds.
    #[must_use]
    #[get]
    pub fn duration(&self) -> f64 {
        Duration::from_millis(self.data.metadata.duration_ms).as_secs_f64()
    }

    /// Returns when this macro was recorded.
    /// @returns Date
    #[get]
    pub fn recorded_at<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let recorded_at = self.data.metadata.recorded_at.into();
        date_from_system_time(&ctx, &recorded_at)
    }

    /// Returns the platform on which this macro was recorded (`"linux"` or `"windows"`).
    #[must_use]
    #[get]
    pub fn platform(&self) -> String {
        self.data.metadata.platform.clone()
    }

    /// Returns a string representation of this macro.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        let duration = Duration::from_millis(self.data.metadata.duration_ms);
        display_with_type(
            "Macro",
            format!(
                "{} events, {}",
                self.event_count(),
                format_duration(duration)
            ),
        )
    }
}

/// Progress of a `macros.play()` operation.
///
/// Received by iterating over the async iterator returned by `play`.
///
/// ```ts
/// const task = macros.play(macro);
/// for await (const progress of task) {
///     console.println(`${Math.round(progress.ratio() * 100)}%`);
///     if (progress.finished()) break;
/// }
/// await task;
/// ```
#[derive(Clone, Copy, Debug, Default, JsLifetime)]
#[js_class]
pub struct JsPlayProgress {
    inner: PlayProgress,
}

impl<'js> Trace<'js> for JsPlayProgress {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl HostClass<'_> for JsPlayProgress {}

impl From<PlayProgress> for JsPlayProgress {
    fn from(inner: PlayProgress) -> Self {
        Self { inner }
    }
}

#[js_methods]
impl JsPlayProgress {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(Exception::throw_message(
            &ctx,
            "PlayProgress cannot be instantiated directly",
        ))
    }

    /// Number of events replayed so far.
    #[must_use]
    pub const fn events_done(&self) -> u64 {
        self.inner.events_done
    }

    /// Total number of events to replay.
    #[must_use]
    pub const fn total_events(&self) -> u64 {
        self.inner.total_events
    }

    /// Replay ratio, in the range `[0, 1]`.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        if self.inner.total_events == 0 {
            return 0.0;
        }
        #[allow(clippy::as_conversions)]
        let ratio = self.inner.events_done as f64 / self.inner.total_events as f64;
        ratio
    }

    /// Whether all events have been replayed.
    #[must_use]
    pub const fn finished(&self) -> bool {
        self.inner.events_done >= self.inner.total_events
    }

    /// Returns a string representation of this playback progress.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "PlayProgress",
            format!("{}/{}", self.inner.events_done, self.inner.total_events),
        )
    }
}

/// Options for `macros.record()`.
///
/// ```ts
/// const m = await macros.record({
///     stopKeys: [Key.Escape],
///     mousePositionInterval: "16ms",
/// });
/// ```
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsRecordOptions {
    /// Key combination that stops the recording.
    /// All listed keys must be pressed simultaneously.
    #[default(vec![JsKey::Standard(JsStandardKey::Escape)], ts = "[Key.Escape]")]
    pub stop_keys: Vec<JsKey>,

    /// Maximum recording duration before automatically stopping.
    pub timeout: Option<JsDuration>,

    /// How often to sample the mouse position.
    #[default(Duration::from_millis(16).into(), ts = "\"16ms\"")]
    pub mouse_position_interval: JsDuration,

    /// Record mouse button press and release events.
    #[default(true)]
    pub mouse_buttons: bool,

    /// Record mouse cursor position.
    #[default(true)]
    pub mouse_position: bool,

    /// Record mouse scroll wheel events.
    #[default(true)]
    pub mouse_scroll: bool,

    /// Record keyboard key press and release events.
    #[default(true)]
    pub keyboard_keys: bool,

    /// Abort signal to cancel recording.
    pub signal: Option<JsAbortSignal>,
}

/// Options for `macros.play()`.
///
/// ```ts
/// await macros.play(macro, { speed: 2.0 });
/// ```
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsPlayOptions {
    /// Playback speed multiplier. `1.0` is real-time, `2.0` is twice as fast.
    /// Must be greater than zero.
    #[default(1.0)]
    pub speed: f64,

    /// Replay mouse button events.
    #[default(true)]
    pub mouse_buttons: bool,

    /// Replay mouse cursor movements.
    #[default(true)]
    pub mouse_position: bool,

    /// Replay mouse movements relative to the current cursor position instead of absolute
    /// screen coordinates. The offset is computed from the difference between the cursor's
    /// position at playback start and the first recorded mouse position.
    pub relative_mouse_position: bool,

    /// Replay mouse scroll events.
    #[default(true)]
    pub mouse_scroll: bool,

    /// Replay keyboard key events.
    #[default(true)]
    pub keyboard_keys: bool,

    /// Abort signal to cancel playback.
    pub signal: Option<JsAbortSignal>,
}

impl From<JsPlayOptions> for PlayConfig {
    fn from(options: JsPlayOptions) -> Self {
        Self {
            speed: options.speed,
            mouse_buttons: options.mouse_buttons,
            mouse_position: options.mouse_position,
            relative_mouse_position: options.relative_mouse_position,
            mouse_scroll: options.mouse_scroll,
            keyboard_keys: options.keyboard_keys,
        }
    }
}

impl Default for PlayConfig {
    fn default() -> Self {
        Self::from(JsPlayOptions::default())
    }
}

/// Records and replays input macros.
///
/// ```ts
/// // Record until Escape is pressed
/// const m = await macros.record();
/// await macros.play(m);
/// ```
///
/// ```ts
/// // Save and reload a macro
/// const m = await macros.record({ timeout: "30s" });
/// await m.save("workflow.amac");
/// const loaded = await Macro.load("workflow.amac");
/// await macros.play(loaded, { speed: 2.0 });
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsMacros {
    runtime: Arc<Runtime>,
    mouse: Mouse,
    macro_player: Arc<MacroPlayer>,
}

impl<'js> Trace<'js> for JsMacros {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsMacros {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let target = registration_target(ctx);
        Class::<JsMacro>::define(&target)?;
        Class::<JsPlayProgress>::define(&target)?;
        Ok(())
    }
}

impl JsMacros {
    /// @skip
    #[instrument(skip_all)]
    pub async fn new(
        runtime: Arc<Runtime>,
        mouse: Mouse,
        macro_player: Arc<MacroPlayer>,
    ) -> super::Result<Self> {
        Ok(Self {
            runtime,
            mouse,
            macro_player,
        })
    }
}

#[js_methods]
impl JsMacros {
    /// Records user input until the stop key combination is pressed (or the timeout elapses).
    ///
    /// ```ts
    /// // Record with default settings (stop with Escape)
    /// const m = await macros.record();
    /// ```
    ///
    /// ```ts
    /// // Record with a 30-second timeout
    /// const m = await macros.record({ timeout: "30s" });
    /// ```
    ///
    /// ```ts
    /// // Record only keyboard events
    /// const m = await macros.record({
    ///     mouseButtons: false,
    ///     mousePosition: false,
    ///     mouseScroll: false,
    /// });
    /// ```
    /// @returns Task<Macro>
    #[platform(not = "wayland")]
    pub fn record<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsRecordOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let runtime = self.runtime.clone();
        let mouse = self.mouse.clone();

        let displays = ctx.user_data().displays();

        let stop_keys = options
            .stop_keys
            .iter()
            .copied()
            .map(|key| {
                enigo::Key::try_from(key).map_err(|_| {
                    Exception::throw_message(
                        &ctx,
                        &format!("stop key {key} is not supported on this platform"),
                    )
                })
            })
            .collect::<Result<HashSet<_>>>()?;

        let config = RecordConfig {
            stop_keys,
            timeout: options.timeout.map(|dur| dur.0),
            mouse_position_interval: options.mouse_position_interval.0,
            mouse_buttons: options.mouse_buttons,
            mouse_position: options.mouse_position,
            mouse_scroll: options.mouse_scroll,
            keyboard_keys: options.keyboard_keys,
        };

        task_with_token(ctx, signal, async move |ctx, token| {
            record_impl(runtime, mouse, config, displays, token)
                .await
                .map(|data| JsMacro {
                    data: Arc::new(data),
                })
                .into_js_result(&ctx)
        })
    }

    /// Replays a previously recorded macro.
    ///
    /// Starting a new playback cancels any previous one before the new macro starts.
    ///
    /// ```ts
    /// await macros.play(macro);
    /// ```
    ///
    /// ```ts
    /// // Play at twice the original speed, skipping mouse movements
    /// await macros.play(macro, { speed: 2.0, mousePosition: false });
    /// ```
    ///
    /// ```ts
    /// // Cancellable playback with progress tracking
    /// const controller = new AbortController();
    /// const task = macros.play(macro, { signal: controller.signal });
    /// for await (const progress of task) {
    ///     console.println(`${Math.round(progress.ratio() * 100)}%`);
    ///     if (progress.finished()) break;
    /// }
    /// await task;
    /// ```
    ///
    /// ```ts
    /// // Starting a second playback cancels the first
    /// const first = macros.play(macroA);
    /// const second = macros.play(macroB);
    /// await second;
    /// ```
    /// @returns ProgressTask<void, PlayProgress>
    #[platform(not = "wayland")]
    pub fn play<'js>(
        &self,
        ctx: Ctx<'js>,
        r#macro: JsMacro,
        options: Opt<JsPlayOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();

        if options.speed <= 0.0 {
            return Err(Exception::throw_message(
                &ctx,
                "play: speed must be greater than zero",
            ));
        }

        let signal = options.signal.clone();
        let macro_data = r#macro.data();
        let player = self.macro_player.clone();
        let config = PlayConfig::from(options);

        let (progress_tx, progress_rx) = mpsc::unbounded_channel::<PlayProgress>();

        progress_task_with_token::<_, _, _, _, _, JsPlayProgress>(
            ctx,
            signal,
            progress_rx,
            async move |ctx, token| {
                player
                    .play(macro_data, config, progress_tx, token)
                    .await
                    .into_js_result(&ctx)
            },
        )
    }

    /// Returns a string representation of the `macros` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Macros".to_string()
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::JsPlayProgress;
    use crate::runtime::Runtime;

    #[test]
    fn empty_play_progress_is_done() {
        let progress = JsPlayProgress::from(super::PlayProgress::default());
        assert!(progress.finished());
    }

    /// Manual test: record a macro, save, reload, and play back.
    /// Press Escape to stop recording.
    #[test]
    #[traced_test]
    #[ignore]
    fn test_record_and_play() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Recording… press Escape to stop.");
                    const m = await macros.record({ mousePositionInterval: "16ms" });
                    console.println(`Recorded ${m.eventCount()} events over ${m.duration()}s.`);
                    console.println("Replaying in 2 seconds…");
                    await sleep("2s");
                    await macros.play(m);
                    console.println("Playback done.");
                "#,
                )
                .await
                .unwrap();
        });
    }

    /// Manual test: record, save to /tmp, reload, play back.
    #[test]
    #[traced_test]
    #[ignore]
    fn test_save_and_load() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Recording… press Escape to stop.");
                    const m = await macros.record();
                    const path = "/tmp/test_macro.amac";
                    await m.save(path);
                    console.println(`Saved to ${path}`);
                    const loaded = await Macro.load(path);
                    console.println(`Loaded: ${loaded.eventCount()} events`);
                    console.println("Playing back…");
                    await macros.play(loaded);
                    console.println("Done.");
                "#,
                )
                .await
                .unwrap();
        });
    }

    /// Manual test: verify playback speed scaling.
    #[test]
    #[traced_test]
    #[ignore]
    fn test_playback_speed() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Recording for 5s (press Escape to stop early)…");
                    const m = await macros.record({ timeout: "5s" });
                    console.println(`Recorded ${m.eventCount()} events.`);
                    console.println("Playing at 2× speed…");
                    await macros.play(m, { speed: 2.0 });
                    console.println("Done.");
                "#,
                )
                .await
                .unwrap();
        });
    }

    /// Manual test: keyboard-only recording.
    #[test]
    #[traced_test]
    #[ignore]
    fn test_keyboard_only() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Recording keyboard only… press Escape to stop.");
                    const m = await macros.record({
                        mouseButtons: false,
                        mousePosition: false,
                        mouseScroll: false,
                    });
                    console.println(`Recorded ${m.eventCount()} keyboard events.`);
                    await macros.play(m);
                    console.println("Done.");
                "#,
                )
                .await
                .unwrap();
        });
    }

    /// Manual test: starting a new playback cancels the previous one.
    #[test]
    #[traced_test]
    #[ignore]
    fn test_concurrent_play_cancels_previous() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Recording a short macro (press Escape)…");
                    const m = await macros.record({ timeout: "3s" });
                    console.println("Starting a slow playback, then restarting it.");
                    const first = macros.play(m, { speed: 0.1 });
                    await sleep("500ms");
                    const second = macros.play(m);

                    try {
                        await first;
                        console.println("First playback unexpectedly completed.");
                    } catch (e) {
                        console.println(`First playback cancelled as expected: ${e.message}`);
                    }

                    await second;
                    console.println("Second playback completed.");
                "#,
                )
                .await
                .unwrap();
        });
    }
}
