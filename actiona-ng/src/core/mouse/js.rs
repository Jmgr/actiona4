use std::{sync::Arc, thread::sleep};

use convert_case::{Case, Casing};
use macros::ExposeEnum;
use rquickjs::{
    Class, Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use strum::Display;

use super::Coordinate;
use crate::{
    IntoJS,
    core::{
        SingletonClass,
        js::ms_to_duration,
        point::js::{JsPoint, JsPointParam},
    },
    runtime::Runtime,
};

impl<T> IntoJS<T> for super::Result<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for super::Mouse {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        self.pressed_buttons
            .iter()
            .for_each(|button| button.trace(tracer));
    }
}

/// Mouse button.
#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, Hash, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "Button")]
pub enum JsButton {
    /// Left button
    Left,

    /// Middle button
    Middle,

    /// Right button
    Right,

    /// Back button
    Back,

    /// Forward button
    Forward,
}

#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "Axis")]
pub enum JsAxis {
    Horizontal,
    Vertical,
}

/// Tweening functions for smooth movement.
#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, Hash, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "Tween")]
pub enum JsTween {
    /// Starts slowly, then accelerates with an overshoot.
    BackIn,
    /// Starts and ends with an overshoot, accelerating in between.
    BackInOut,
    /// Starts quickly, then decelerates with an overshoot.
    BackOut,

    /// Starts by bouncing off the start point.
    BounceIn,
    /// Bounces at both the start and end points.
    BounceInOut,
    /// Ends with a bounce effect.
    BounceOut,

    /// Starts slowly and accelerates in a circular motion.
    CircIn,
    /// Starts and ends slowly with a circular motion.
    CircInOut,
    /// Ends slowly with a circular motion.
    CircOut,

    /// Starts slowly and accelerates cubically.
    CubicIn,
    /// Starts and ends slowly with a cubic acceleration.
    CubicInOut,
    /// Ends slowly with a cubic deceleration.
    CubicOut,

    /// Starts with an elastic effect, overshooting the target.
    ElasticIn,
    /// Starts and ends with an elastic effect.
    ElasticInOut,
    /// Ends with an elastic effect, overshooting the target.
    ElasticOut,

    /// Starts slowly and accelerates exponentially.
    ExpoIn,
    /// Starts and ends slowly with an exponential acceleration.
    ExpoInOut,
    /// Ends slowly with an exponential deceleration.
    ExpoOut,

    /// A linear tween with no acceleration or deceleration.
    Linear,

    /// Starts slowly and accelerates quadratically.
    QuadIn,
    /// Starts and ends slowly with a quadratic acceleration.
    QuadInOut,
    /// Ends slowly with a quadratic deceleration.
    QuadOut,

    /// Starts slowly and accelerates quartically.
    QuartIn,
    /// Starts and ends slowly with a quartic acceleration.
    QuartInOut,
    /// Ends slowly with a quartic deceleration.
    QuartOut,

    /// Starts slowly and accelerates quintically.
    QuintIn,
    /// Starts and ends slowly with a quintic acceleration.
    QuintInOut,
    /// Ends slowly with a quintic deceleration.
    QuintOut,

    /// Starts slowly and accelerates sinusoidally.
    SineIn,
    /// Starts and ends slowly with a sinusoidal acceleration.
    SineInOut,
    /// Ends slowly with a sinusoidal deceleration.
    SineOut,
}

/// @global
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Mouse")]
pub struct JsMouse {
    inner: super::Mouse,
}

impl<'js> SingletonClass<'js> for JsMouse {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        JsButton::register(ctx)?;
        JsAxis::register(ctx)?;

        Ok(())
    }
}

impl JsMouse {
    /// @skip
    pub async fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        Ok(Self {
            inner: super::Mouse::new(runtime).await?,
        })
    }
}

pub type JsMoveOptions = super::MoveOptions;
pub type JsPressOptions = super::PressOptions;
pub type JsClickOptions = super::ClickOptions;
pub type JsDoubleClickOptions = super::DoubleClickOptions;

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMouse {
    pub fn is_pressed(&mut self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        self.inner.is_pressed(button).into_js(&ctx)
    }

    pub fn scroll<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'_>,
        length: i32,
        axis: Opt<JsAxis>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .scroll(length, axis.unwrap_or(JsAxis::Vertical))
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn position(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self.inner.position().into_js(&ctx)?.into())
    }

    pub fn measure_speed(&self, ctx: Ctx<'_>, duration: Opt<f64>) -> Result<f32> {
        let duration = ms_to_duration(duration.unwrap_or(2000.));
        self.inner.measure_speed(duration).into_js(&ctx)
    }

    #[qjs(rename = "move")]
    pub fn r#move<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        point: JsPointParam,
        options: Opt<JsMoveOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .move_(point.0, options.unwrap_or_default())
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn set_position<'js>(
        &self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        point: JsPointParam,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .set_position(point.0, Coordinate::Abs)
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn set_relative_position<'js>(
        &self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        point: JsPointParam,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .set_position(point.0, Coordinate::Rel)
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn click<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'_>,
        options: Opt<JsClickOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .click(options.unwrap_or_default())
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn double_click<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'_>,
        options: Opt<JsDoubleClickOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .double_click(options.unwrap_or_default())
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn press<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'_>,
        options: Opt<JsPressOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .press(options.unwrap_or_default())
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn release<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'_>,
        button: Opt<JsButton>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .release(button.map(|button| button))
            .into_js(&ctx)?;

        Ok(this.0)
    }

    pub fn wait<'js>(
        &self,
        this: This<Class<'js, Self>>,
        duration: f64,
    ) -> Result<Class<'js, Self>> {
        sleep(ms_to_duration(duration));

        Ok(this.0)
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
            let mut position: JsPoint = script_engine.eval("mouse.position()").await.unwrap();
            position = point(position.get_x() + 5, position.get_y() + 5).into();

            script_engine
                .eval::<()>(&format!(
                    "mouse.setPosition(new Point{})",
                    position.to_string_js()
                ))
                .await
                .unwrap();

            script_engine
                .eval::<()>(&format!(
                    "mouse.setPosition({}, {})",
                    position.get_x(),
                    position.get_y()
                ))
                .await
                .unwrap();

            script_engine
                .eval::<()>(&format!(
                    "mouse.setPosition({{ x: {}, y: {} }})",
                    position.get_x(),
                    position.get_y()
                ))
                .await
                .unwrap();

            let new_position: JsPoint = script_engine.eval("mouse.position()").await.unwrap();
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
                .eval("mouse.isPressed(Button.LEFT)")
                .await
                .unwrap();
            assert!(pressed);

            script_engine.eval::<()>("mouse.release()").await.unwrap();

            let pressed: bool = script_engine
                .eval("mouse.isPressed(Button.LEFT)")
                .await
                .unwrap();
            assert!(!pressed);
        });
    }

    #[test]
    #[traced_test]
    fn test_scroll() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine.eval::<()>("mouse.scroll(1)").await.unwrap();
            script_engine.eval::<()>("mouse.scroll(-1)").await.unwrap();

            script_engine
                .eval::<()>("mouse.scroll(1, Axis.HORIZONTAL)")
                .await
                .unwrap();
            script_engine
                .eval::<()>("mouse.scroll(-1, Axis.HORIZONTAL)")
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    fn test_measure_speed() {
        Runtime::test_with_script_engine(async |script_engine| {
            let speed: f64 = script_engine
                .eval("mouse.measureSpeed(2000)")
                .await
                .unwrap();
            println!("speed: {speed}");
        });
    }

    #[test]
    #[traced_test]
    fn test_wait() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval::<()>("mouse.wait(100).wait(200);")
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    fn test_chain() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval::<()>(
                    r#"
mouse.move(2000, 1000).wait(2000).move(3000, 800);
            "#,
                )
                .await
                .unwrap();
        });
    }
}
