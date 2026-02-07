use std::{sync::Arc, time::Duration};

use macros::FromJsObject;
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use tracing::instrument;

use super::Coordinate;
use crate::{
    IntoJsResult,
    core::{
        js::{
            abort_controller::JsAbortSignal,
            classes::{SingletonClass, register_enum},
            duration::{JsDuration, secs_to_duration},
            task::{task, task_with_token},
        },
        point::js::{JsPoint, JsPointLike},
    },
    runtime::{Runtime, WithUserData},
};

impl<T> IntoJsResult<T> for super::Result<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

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
/// await mouse.move(new Point(500, 300));
/// await mouse.click();
/// ```
///
/// ```ts
/// // Right-click at a specific position
/// await mouse.click({ button: Button.Right, position: new Point(100, 200) });
/// ```
///
/// ```ts
/// // Smooth movement with custom tween
/// await mouse.move(new Point(800, 600), {
///   speed: 1500,
///   tween: Tween.BounceOut
/// });
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Mouse")]
pub struct JsMouse {
    inner: Arc<super::Mouse>,
}

impl<'js> SingletonClass<'js> for JsMouse {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsButton>(ctx)?;
        register_enum::<JsAxis>(ctx)?;

        Ok(())
    }
}

impl<'js> Trace<'js> for JsMouse {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsMouse {
    /// @skip
    #[instrument(skip_all)]
    pub async fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        Ok(Self {
            inner: Arc::new(super::Mouse::new(runtime).await?),
        })
    }
}

pub type JsMoveOptions = super::MoveOptions;
pub type JsPressOptions = super::PressOptions;

/// Options for measuring mouse movement speed.
///
/// ```ts
/// const speed = await mouse.measureSpeed({ duration: 3 });
/// ```
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsMeasureSpeedOptions {
    /// Duration in seconds
    /// @default `2`
    pub duration: Option<f64>,

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

    /// @default `1`
    pub amount: i32,

    /// @default `0`
    pub interval: JsDuration,

    /// @default `0`
    pub duration: JsDuration,

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
    pub async fn is_pressed(&self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        self.inner.is_pressed(button).await.into_js_result(&ctx)
    }

    /// Scrolls the mouse wheel by the given amount.
    pub async fn scroll(&self, ctx: Ctx<'_>, length: i32, axis: Opt<JsAxis>) -> Result<()> {
        self.inner
            .scroll(length, axis.unwrap_or(JsAxis::Vertical))
            .into_js_result(&ctx)
    }

    /// Returns the current mouse cursor position.
    /// @platforms -wayland
    pub async fn position(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
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
        let duration = secs_to_duration(options.duration.unwrap_or(2.));
        let local_mouse = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_mouse
                .measure_speed(duration, token)
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
    pub async fn set_position(&self, ctx: Ctx<'_>, point: JsPointLike) -> Result<()> {
        self.inner
            .set_position(point.0, Coordinate::Abs)
            .into_js_result(&ctx)
    }

    /// Moves the mouse cursor by the given offset (relative coordinates).
    pub async fn set_relative_position(&self, ctx: Ctx<'_>, point: JsPointLike) -> Result<()> {
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
    pub async fn press(&self, ctx: Ctx<'_>, options: Opt<JsPressOptions>) -> Result<()> {
        self.inner
            .press(options.unwrap_or_default())
            .into_js_result(&ctx)
    }

    /// Releases a mouse button.
    pub async fn release(&self, ctx: Ctx<'_>, button: Opt<JsButton>) -> Result<()> {
        self.inner
            .release(button.map(|button| button))
            .into_js_result(&ctx)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::JsButton;
    use crate::{
        core::point::{js::JsPoint, point},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    #[ignore]
    fn test_position() {
        Runtime::test_with_script_engine(async |script_engine| {
            let mut position: JsPoint = script_engine
                .eval_async("await mouse.position()")
                .await
                .unwrap();
            position = point(position.get_x() + 5, position.get_y() + 5).into();

            script_engine
                .eval_async::<()>(&format!(
                    "await mouse.setPosition(new Point{})",
                    position.to_string_js()
                ))
                .await
                .unwrap();

            script_engine
                .eval_async::<()>(&format!(
                    "await mouse.setPosition({}, {})",
                    position.get_x(),
                    position.get_y()
                ))
                .await
                .unwrap();

            script_engine
                .eval_async::<()>(&format!(
                    "await mouse.setPosition({{ x: {}, y: {} }})",
                    position.get_x(),
                    position.get_y()
                ))
                .await
                .unwrap();

            let new_position: JsPoint = script_engine
                .eval_async("await mouse.position()")
                .await
                .unwrap();
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
                .eval_async::<()>("await mouse.scroll(1)")
                .await
                .unwrap();
            script_engine
                .eval_async::<()>("await mouse.scroll(-1)")
                .await
                .unwrap();

            script_engine
                .eval_async::<()>("await mouse.scroll(1, Axis.Horizontal)")
                .await
                .unwrap();
            script_engine
                .eval_async::<()>("await mouse.scroll(-1, Axis.Horizontal)")
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_measure_speed() {
        Runtime::test_with_script_engine(async |script_engine| {
            let speed: f64 = script_engine
                .eval_async("await mouse.measureSpeed(2000)")
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
