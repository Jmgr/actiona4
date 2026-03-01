use std::{sync::Arc, time::Duration};

use macros::FromJsObject;
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::*,
};
use tracing::instrument;

use super::Coordinate;
use crate::{
    IntoJsResult,
    api::{
        js::{
            abort_controller::JsAbortSignal,
            classes::{SingletonClass, register_enum},
            duration::JsDuration,
            event_handle::{HandleId, JsEventHandle},
            task::{task, task_with_token},
        },
        mouse::{
            click_triggers::{ClickTriggers, OnButtonOptions},
            scroll_triggers::ScrollTriggers,
        },
        point::js::{JsPoint, JsPointLike},
    },
    runtime::{Runtime, WithUserData},
    types::display::display_with_type,
};

impl<'js> Trace<'js> for super::Mouse {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

pub type JsButton = super::Button;
pub type JsAxis = super::Axis;
pub type JsTween = super::Tween;

/// Controls mouse input: movement, clicking, scrolling, and position queries.
///
/// ```ts
/// // Move and click
/// await mouse.move(500, 300);
/// await mouse.click();
/// ```
///
/// ```ts
/// // Right-click at a specific position
/// await mouse.click({ button: Button.Right, position: { x: 100, y: 200 } });
/// ```
///
/// ```ts
/// // Smooth movement with custom tween
/// await mouse.move(800, 600, {
///   speed: 1500,
///   tween: Tween.BounceOut
/// });
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Mouse")]
pub struct JsMouse {
    inner: super::Mouse,
    click_triggers: ClickTriggers,
    scroll_triggers: ScrollTriggers,
}

impl<'js> SingletonClass<'js> for JsMouse {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsButton>(ctx)?;
        register_enum::<JsAxis>(ctx)?;
        register_enum::<JsTween>(ctx)?;

        Ok(())
    }
}

impl<'js> Trace<'js> for JsMouse {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> Trace<'js> for ClickTriggers {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> Trace<'js> for ScrollTriggers {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsMouse {
    /// @skip
    #[instrument(skip_all)]
    pub async fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        let click_triggers = ClickTriggers::new(
            runtime.clone(),
            runtime.task_tracker(),
            runtime.cancellation_token(),
        );
        let scroll_triggers = ScrollTriggers::new(
            runtime.clone(),
            runtime.task_tracker(),
            runtime.cancellation_token(),
        );

        Ok(Self {
            inner: super::Mouse::new(runtime).await?,
            click_triggers,
            scroll_triggers,
        })
    }
}

pub type JsMoveOptions = super::MoveOptions;
pub type JsPressOptions = super::PressOptions;

/// Options for `onButton`.
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsOnButtonOptions {
    /// Require exactly this button and no others to be pressed simultaneously.
    /// @default `false`
    pub exclusive: bool,

    /// Abort signal to automatically cancel this listener when signalled.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl From<JsOnButtonOptions> for OnButtonOptions {
    fn from(options: JsOnButtonOptions) -> Self {
        Self {
            exclusive: options.exclusive,
        }
    }
}

/// Options for `onScroll`.
/// @options
#[derive(Clone, Debug, FromJsObject)]
pub struct JsOnScrollOptions {
    /// Axis to listen on.
    /// @default `Axis.Vertical`
    pub axis: super::Axis,

    /// Abort signal to automatically cancel this listener when signalled.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl Default for JsOnScrollOptions {
    fn default() -> Self {
        Self {
            axis: super::Axis::Vertical,
            signal: None,
        }
    }
}

/// Options for measuring mouse movement speed.
///
/// ```ts
/// const speed = await mouse.measureSpeed({ duration: "3s" });
/// ```
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsMeasureSpeedOptions {
    /// Measurement duration.
    /// @default `2s`
    pub duration: Option<JsDuration>,

    /// Abort signal to cancel the measurement.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

/// Options for clicking a mouse button.
///
/// ```ts
/// // Click and hold for 0.5 seconds
/// await mouse.click({ duration: 0.5 });
/// ```
/// @extends PressOptions
/// @options
#[derive(Clone, Debug, FromJsObject)]
pub struct JsClickOptions {
    /// @skip
    pub press: super::PressOptions,

    /// Number of times to click.
    /// @default `1`
    pub amount: i32,

    /// Delay between consecutive clicks, in seconds.
    /// @default `0`
    pub interval: JsDuration,

    /// How long to hold each click, in seconds.
    /// @default `0`
    pub duration: JsDuration,

    /// Abort signal to cancel the click.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl Default for JsClickOptions {
    fn default() -> Self {
        Self {
            press: super::PressOptions::default(),
            amount: 1,
            interval: Duration::ZERO.into(),
            duration: Duration::ZERO.into(),
            signal: None,
        }
    }
}

impl JsClickOptions {
    fn into_inner(self) -> super::ClickOptions {
        super::ClickOptions {
            press: self.press,
            amount: self.amount,
            interval: self.interval,
            duration: self.duration,
        }
    }
}

/// Options for double-clicking a mouse button.
///
/// ```ts
/// await mouse.doubleClick({ delay: 0.1 });
/// ```
/// @extends ClickOptions
/// @options
#[derive(Clone, Debug, FromJsObject)]
pub struct JsDoubleClickOptions {
    /// @skip
    pub click: JsClickOptions,

    /// Delay between the two clicks, in seconds.
    /// @default `0.25`
    pub delay: JsDuration,
}

impl Default for JsDoubleClickOptions {
    fn default() -> Self {
        Self {
            click: JsClickOptions::default(),
            delay: Duration::from_millis(250).into(),
        }
    }
}

impl JsDoubleClickOptions {
    fn into_inner(self) -> super::DoubleClickOptions {
        super::DoubleClickOptions {
            click: self.click.into_inner(),
            delay: self.delay,
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMouse {
    /// Returns whether a mouse button is currently pressed.
    /// @platforms -wayland
    pub fn is_pressed(&self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        self.inner.is_pressed(button).into_js_result(&ctx)
    }

    /// Scrolls the mouse wheel by the given amount.
    pub fn scroll(&self, ctx: Ctx<'_>, length: i32, axis: Opt<JsAxis>) -> Result<()> {
        self.inner
            .scroll(length, axis.unwrap_or(JsAxis::Vertical))
            .into_js_result(&ctx)
    }

    /// Returns the current mouse cursor position.
    /// @platforms -wayland
    /// @readonly
    pub fn position(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self.inner.position().into_js_result(&ctx)?.into())
    }

    /// Measures the mouse movement speed over a duration (in pixels per second).
    /// @returns Task<number>
    pub fn measure_speed<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsMeasureSpeedOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let duration = options
            .duration
            .unwrap_or_else(|| Duration::from_secs(2).into());
        let local_mouse = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_mouse
                .measure_speed(duration.0, token)
                .await
                .into_js_result(&ctx)
        })
    }

    /// Moves the mouse cursor smoothly to the given position.
    /// @returns Task<void>
    #[qjs(rename = "move")]
    pub fn r#move<'js>(
        &self,
        ctx: Ctx<'js>,
        point: JsPointLike,
        options: Opt<JsMoveOptions>,
    ) -> Result<Promise<'js>> {
        let local_mouse = self.inner.clone();

        task(ctx, async move |ctx, token| {
            local_mouse
                .move_(
                    point.0,
                    token,
                    options.unwrap_or_default(),
                    ctx.user_data().rng(),
                )
                .await
                .into_js_result(&ctx)
        })
    }

    /// Sets the mouse cursor position instantly (absolute coordinates).
    pub fn set_position(&self, ctx: Ctx<'_>, point: JsPointLike) -> Result<()> {
        self.inner
            .set_position(point.0, Coordinate::Abs)
            .into_js_result(&ctx)
    }

    /// Moves the mouse cursor by the given offset (relative coordinates).
    pub fn set_relative_position(&self, ctx: Ctx<'_>, point: JsPointLike) -> Result<()> {
        self.inner
            .set_position(point.0, Coordinate::Rel)
            .into_js_result(&ctx)
    }

    /// Clicks a mouse button.
    /// @returns Task<void>
    pub fn click<'js>(&self, ctx: Ctx<'js>, options: Opt<JsClickOptions>) -> Result<Promise<'js>> {
        let local_mouse = self.inner.clone();
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_mouse
                .click(options.into_inner(), token)
                .await
                .into_js_result(&ctx)
        })
    }

    /// Double-clicks a mouse button.
    /// @returns Task<void>
    pub fn double_click<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsDoubleClickOptions>,
    ) -> Result<Promise<'js>> {
        let local_mouse = self.inner.clone();
        let options = options.0.unwrap_or_default();
        let signal = options.click.signal.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_mouse
                .double_click(options.into_inner(), token)
                .await
                .into_js_result(&ctx)
        })
    }

    /// Presses and holds a mouse button.
    pub fn press(&self, ctx: Ctx<'_>, options: Opt<JsPressOptions>) -> Result<()> {
        self.inner
            .press(options.unwrap_or_default())
            .into_js_result(&ctx)
    }

    /// Releases a mouse button.
    pub fn release(&self, ctx: Ctx<'_>, button: Opt<JsButton>) -> Result<()> {
        self.inner
            .release(button.map(|button| button))
            .into_js_result(&ctx)
    }

    /// Registers a listener that fires when a mouse button is pressed.
    ///
    /// ```ts
    /// const handle = mouse.onButton(Button.Left, () => {
    ///   console.println("Left button pressed!");
    /// });
    /// // ... later:
    /// handle.cancel();
    /// ```
    ///
    /// @param button: Button
    /// @param callback: () => void | Promise<void>
    /// @param options?: OnButtonOptions
    /// @returns EventHandle
    pub fn on_button<'js>(
        &self,
        ctx: Ctx<'js>,
        button: JsButton,
        callback: Value<'js>,
        options: Opt<JsOnButtonOptions>,
    ) -> Result<JsEventHandle> {
        let Some(function) = callback.as_function() else {
            return Err(Exception::throw_type(&ctx, "callback must be a function"));
        };

        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let button_options = options.into();
        let id = HandleId::default();
        let user_data = ctx.user_data();
        let function_key = user_data.callbacks().register(&ctx, function.clone());
        let context = user_data.script_engine().context();
        self.click_triggers
            .add(id, button, context, function_key, button_options);

        let handle = JsEventHandle::new(id, Arc::new(self.click_triggers.clone()));
        Self::cancel_handle_on_signal(&ctx, signal, handle.clone());

        Ok(handle)
    }

    /// Registers a listener that fires when the mouse wheel is scrolled.
    ///
    /// ```ts
    /// const handle = mouse.onScroll((length) => {
    ///   console.println(`Scrolled ${length} units`);
    /// });
    /// // ... later:
    /// handle.cancel();
    /// ```
    ///
    /// ```ts
    /// // Listen for horizontal scroll only
    /// const handle = mouse.onScroll((length) => {
    ///   console.println(`Horizontal scroll: ${length}`);
    /// }, { axis: Axis.Horizontal });
    /// ```
    ///
    /// @param callback: (length: number) => void | Promise<void>
    /// @param options?: OnScrollOptions
    /// @returns EventHandle
    pub fn on_scroll<'js>(
        &self,
        ctx: Ctx<'js>,
        callback: Value<'js>,
        options: Opt<JsOnScrollOptions>,
    ) -> Result<JsEventHandle> {
        let Some(function) = callback.as_function() else {
            return Err(Exception::throw_type(&ctx, "callback must be a function"));
        };

        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let id = HandleId::default();
        let user_data = ctx.user_data();
        let function_key = user_data.callbacks().register(&ctx, function.clone());
        let context = user_data.script_engine().context();
        self.scroll_triggers
            .add(id, options.axis, context, function_key);

        let handle = JsEventHandle::new(id, Arc::new(self.scroll_triggers.clone()));
        Self::cancel_handle_on_signal(&ctx, signal, handle.clone());

        Ok(handle)
    }

    /// Unregisters all event handles registered on this mouse instance.
    ///
    /// ```ts
    /// mouse.onButton(Button.Left, () => console.println("left"));
    /// mouse.clearEventHandles();
    /// ```
    pub fn clear_event_handles(&self) {
        self.click_triggers.clear();
        self.scroll_triggers.clear();
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Mouse", &self.inner)
    }
}

impl JsMouse {
    fn cancel_handle_on_signal(
        ctx: &Ctx<'_>,
        signal: Option<JsAbortSignal>,
        handle: JsEventHandle,
    ) {
        let Some(signal) = signal else {
            return;
        };

        let token = signal.into_token();
        ctx.user_data().task_tracker().spawn(async move {
            token.cancelled().await;
            handle.cancel();
        });
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::JsButton;
    use crate::{
        api::point::{js::JsPoint, point},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    #[ignore]
    fn test_position() {
        Runtime::test_with_script_engine(async |script_engine| {
            let mut position: JsPoint = script_engine.eval_async("mouse.position()").await.unwrap();
            position = point(position.get_x() + 5, position.get_y() + 5).into();

            script_engine
                .eval_async::<()>(&format!(
                    "mouse.setPosition(new Point{})",
                    position.to_string_js()
                ))
                .await
                .unwrap();

            script_engine
                .eval_async::<()>(&format!(
                    "mouse.setPosition({}, {})",
                    position.get_x(),
                    position.get_y()
                ))
                .await
                .unwrap();

            script_engine
                .eval_async::<()>(&format!(
                    "mouse.setPosition({{ x: {}, y: {} }})",
                    position.get_x(),
                    position.get_y()
                ))
                .await
                .unwrap();

            let new_position: JsPoint = script_engine.eval_async("mouse.position()").await.unwrap();
            assert_eq!(position, new_position);
        });
    }

    #[test]
    #[traced_test]
    fn test_button() {
        Runtime::test_with_script_engine(async |script_engine| {
            let button: JsButton = script_engine.eval("Button.Left").await.unwrap();
            assert_eq!(button, JsButton::Left);

            let button: JsButton = script_engine.eval("Button.Right").await.unwrap();
            assert_eq!(button, JsButton::Right);
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_press_release() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine.eval::<()>("mouse.press()").await.unwrap();

            let pressed: bool = script_engine
                .eval_async("await mouse.isPressed(Button.Left)")
                .await
                .unwrap();
            assert!(pressed);

            script_engine.eval::<()>("mouse.release()").await.unwrap();

            let pressed: bool = script_engine
                .eval_async("await mouse.isPressed(Button.Left)")
                .await
                .unwrap();
            assert!(!pressed);
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_scroll() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>("mouse.scroll(1)")
                .await
                .unwrap();
            script_engine
                .eval_async::<()>("mouse.scroll(-1)")
                .await
                .unwrap();

            script_engine
                .eval_async::<()>("mouse.scroll(1, Axis.Horizontal)")
                .await
                .unwrap();
            script_engine
                .eval_async::<()>("mouse.scroll(-1, Axis.Horizontal)")
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_on_button() {
        Runtime::test_with_script_engine(async |script_engine| {
            _ = script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Registering mouse.onButton for left button.");
                    console.println("Press Escape to end this manual test.");
                    const handle = mouse.onButton(Button.Left, async () => {
                        await sleep(250);
                        console.println("Left button pressed");
                    });

                    await keyboard.waitForKeys([Key.Escape]);
                    console.println("STOPPING");
                    handle.cancel();
                    console.println("END");
                "#,
                )
                .await;
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_on_scroll() {
        Runtime::test_with_script_engine(async |script_engine| {
            _ = script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Registering mouse.onScroll for vertical axis.");
                    console.println("Scroll the mouse wheel or press Escape to end this manual test.");
                    const handle = mouse.onScroll((length) => {
                        console.println(`Vertical scroll: ${length}`);
                    });

                    await keyboard.waitForKeys([Key.Escape]);
                    console.println("STOPPING");
                    handle.cancel();
                    console.println("END");
                "#,
                )
                .await;
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_measure_speed() {
        Runtime::test_with_script_engine(async |script_engine| {
            let speed: f64 = script_engine
                .eval_async("await mouse.measureSpeed({ duration: \"2s\" })")
                .await
                .unwrap();
            println!("speed: {speed}");
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_random_move_timeout() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    async function timeout(ms) {
                        await mouse.pause(ms);
                    }
                    async function timeConsumingTask() {
                        while(true) {
                            let pos = displays.randomPoint();
                            await mouse.move(pos);
                        }
                    }
                    Promise.race([
                        timeout(2000),
                        timeConsumingTask(),
                    ]);
                    "#,
                )
                .await
                .unwrap();
        });
    }
}
