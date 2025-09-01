use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};

use super::Coordinate;
use crate::{
    IntoJsResult,
    core::{
        js::{classes::SingletonClass, duration::ms_to_duration, task::task},
        point::js::{JsPoint, JsPointParam},
    },
    runtime::{Runtime, WithUserData},
};

impl<T> IntoJsResult<T> for super::Result<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for super::Mouse {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        let lock = self.pressed_buttons.lock().unwrap();
        lock.iter().for_each(|button| button.trace(tracer));
    }
}

pub type JsButton = super::Button;
pub type JsAxis = super::Axis;
pub type JsTween = super::Tween;

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Mouse")]
pub struct JsMouse {
    inner: Arc<super::Mouse>,
}

impl<'js> SingletonClass<'js> for JsMouse {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        JsButton::register(ctx)?;
        JsAxis::register(ctx)?;

        Ok(())
    }
}

impl<'js> Trace<'js> for JsMouse {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsMouse {
    /// @skip
    pub async fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        Ok(Self {
            inner: Arc::new(super::Mouse::new(runtime).await?),
        })
    }
}

pub type JsMoveOptions = super::MoveOptions;
pub type JsPressOptions = super::PressOptions;
pub type JsClickOptions = super::ClickOptions;
pub type JsDoubleClickOptions = super::DoubleClickOptions;

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMouse {
    /// @platforms -wayland
    pub async fn is_pressed(&self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        self.inner.is_pressed(button).await.into_js(&ctx)
    }

    pub async fn scroll(&self, ctx: Ctx<'_>, length: i32, axis: Opt<JsAxis>) -> Result<()> {
        self.inner
            .scroll(length, axis.unwrap_or(JsAxis::Vertical))
            .into_js(&ctx)
    }

    /// @platforms -wayland
    pub async fn position(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self.inner.position().into_js(&ctx)?.into())
    }

    pub async fn measure_speed(&self, ctx: Ctx<'_>, duration: Opt<f64>) -> Result<f32> {
        let duration = ms_to_duration(duration.unwrap_or(2000.));
        self.inner.measure_speed(duration).await.into_js(&ctx)
    }

    /// @returns Task<void>
    #[qjs(rename = "move")]
    pub fn r#move<'js>(
        &self,
        ctx: Ctx<'js>,
        point: JsPointParam,
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
                .into_js(&ctx)
        })
    }

    pub async fn set_position(&self, ctx: Ctx<'_>, point: JsPointParam) -> Result<()> {
        self.inner
            .set_position(point.0, Coordinate::Abs)
            .into_js(&ctx)
    }

    pub async fn set_relative_position(&self, ctx: Ctx<'_>, point: JsPointParam) -> Result<()> {
        self.inner
            .set_position(point.0, Coordinate::Rel)
            .into_js(&ctx)
    }

    pub async fn click(&self, ctx: Ctx<'_>, options: Opt<JsClickOptions>) -> Result<()> {
        self.inner
            .click(options.unwrap_or_default())
            .await
            .into_js(&ctx)
    }

    pub async fn double_click(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsDoubleClickOptions>,
    ) -> Result<()> {
        self.inner
            .double_click(options.unwrap_or_default())
            .await
            .into_js(&ctx)
    }

    pub async fn press(&self, ctx: Ctx<'_>, options: Opt<JsPressOptions>) -> Result<()> {
        self.inner.press(options.unwrap_or_default()).into_js(&ctx)
    }

    pub async fn release(&self, ctx: Ctx<'_>, button: Opt<JsButton>) -> Result<()> {
        self.inner
            .release(button.map(|button| button))
            .into_js(&ctx)
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
            let button: JsButton = script_engine.eval("Button.LEFT").await.unwrap();
            assert_eq!(button, JsButton::Left);

            let button: JsButton = script_engine.eval("Button.RIGHT").await.unwrap();
            assert_eq!(button, JsButton::Right);
        });
    }

    #[test]
    #[traced_test]
    fn test_press_release() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine.eval::<()>("mouse.press()").await.unwrap();

            let pressed: bool = script_engine
                .eval_async("await mouse.isPressed(Button.LEFT)")
                .await
                .unwrap();
            assert!(pressed);

            script_engine.eval::<()>("mouse.release()").await.unwrap();

            let pressed: bool = script_engine
                .eval_async("await mouse.isPressed(Button.LEFT)")
                .await
                .unwrap();
            assert!(!pressed);
        });
    }

    #[test]
    #[traced_test]
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
                .eval_async::<()>("await mouse.scroll(1, Axis.HORIZONTAL)")
                .await
                .unwrap();
            script_engine
                .eval_async::<()>("await mouse.scroll(-1, Axis.HORIZONTAL)")
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
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
