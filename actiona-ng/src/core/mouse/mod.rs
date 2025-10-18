use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use convert_case::{Case, Casing};
use derive_more::Display;
use enigo::{Direction, Enigo, InputError, NewConError};
use indexmap::IndexSet;
use macros::{ExposeEnum, FromJsObject};
use noiselib::{perlin::perlin_noise_1d, uniform::UniformRandomGen};
use num_traits::ToPrimitive;
use platform::MouseImplTrait;
use rquickjs::{JsLifetime, class::Trace};
use thiserror::Error;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};
use tween::FixedTweener;

use crate::{
    core::{
        js::duration::JsDuration,
        point::{js::JsPoint, try_point},
    },
    error::CommonError,
    runtime::{events::MouseButtonEvent, shared_rng::SharedRng},
};

pub(crate) mod platform;

pub mod js;

pub use enigo::Coordinate;
pub use js::{JsAxis, JsTween};
#[cfg(windows)]
use platform::win::MouseImpl;
#[cfg(unix)]
use platform::x11::MouseImpl;

use super::point::Point;
use crate::{core::point::point, runtime::Runtime};

#[derive(Debug, Error)]
pub enum MouseError {
    #[error(transparent)]
    CommonError(#[from] CommonError),

    #[error(transparent)]
    EyreReport(#[from] eyre::Report),

    #[error("Connecting to the X11 server failed: {0}")]
    ConnectError(String),

    #[error("Connection to the X11 server failed: {0}")]
    ConnectionError(String),

    #[error("X11 reply error: {0}")]
    ReplyError(String),

    #[error("Could not find master pointer device")]
    NoMasterPointerDevice,

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Enigo new connection error: {0}")]
    EnigoNewConnError(#[from] NewConError),

    #[error("Enigo input error: {0}")]
    EnigoInputError(#[from] InputError),

    #[error("{0}")]
    ParameterError(String),
}

pub type Result<T> = std::result::Result<T, MouseError>;

/// Mouse button.
#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, Hash, JsLifetime, PartialEq, Trace)]
#[rquickjs::class]
pub enum Button {
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

impl From<Button> for enigo::Button {
    fn from(value: Button) -> Self {
        use Button::*;

        match value {
            Left => enigo::Button::Left,
            Middle => enigo::Button::Middle,
            Right => enigo::Button::Right,
            Back => enigo::Button::Back,
            Forward => enigo::Button::Forward,
        }
    }
}

#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl From<Axis> for enigo::Axis {
    fn from(value: Axis) -> Self {
        use Axis::*;

        match value {
            Horizontal => enigo::Axis::Horizontal,
            Vertical => enigo::Axis::Vertical,
        }
    }
}

/// Tweening functions for smooth movement.
#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, Hash, JsLifetime, PartialEq, Trace)]
#[rquickjs::class]
pub enum Tween {
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

impl Tween {
    fn into_tween<Value: tween::TweenValue>(self) -> Box<dyn tween::Tween<Value> + Send> {
        use Tween::*;

        match self {
            Linear => Box::new(tween::Linear),
            BackIn => Box::new(tween::BackIn),
            BackInOut => Box::new(tween::BackInOut),
            BackOut => Box::new(tween::BackOut),
            BounceIn => Box::new(tween::BounceIn),
            BounceInOut => Box::new(tween::BounceInOut),
            BounceOut => Box::new(tween::BounceOut),
            CircIn => Box::new(tween::CircIn),
            CircInOut => Box::new(tween::CircInOut),
            CircOut => Box::new(tween::CircOut),
            CubicIn => Box::new(tween::CubicIn),
            CubicInOut => Box::new(tween::CubicInOut),
            CubicOut => Box::new(tween::CubicOut),
            ElasticIn => Box::new(tween::ElasticIn),
            ElasticInOut => Box::new(tween::ElasticInOut),
            ElasticOut => Box::new(tween::ElasticOut),
            ExpoIn => Box::new(tween::ExpoIn),
            ExpoInOut => Box::new(tween::ExpoInOut),
            ExpoOut => Box::new(tween::ExpoOut),
            QuadIn => Box::new(tween::QuadIn),
            QuadInOut => Box::new(tween::QuadInOut),
            QuadOut => Box::new(tween::QuadOut),
            QuartIn => Box::new(tween::QuartIn),
            QuartInOut => Box::new(tween::QuartInOut),
            QuartOut => Box::new(tween::QuartOut),
            QuintIn => Box::new(tween::QuintIn),
            QuintInOut => Box::new(tween::QuintInOut),
            QuintOut => Box::new(tween::QuintOut),
            SineIn => Box::new(tween::SineIn),
            SineInOut => Box::new(tween::SineInOut),
            SineOut => Box::new(tween::SineOut),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ButtonConditions {
    button: Option<Button>,
    direction: Option<Direction>,
}

#[derive(Debug)]
pub struct Mouse {
    enigo: Arc<Mutex<Enigo>>,
    implementation: MouseImpl,
    pressed_buttons: Mutex<IndexSet<Button>>,
}

// TODO: record
// TODO: drag and drop?

impl Mouse {
    #[instrument]
    pub async fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self {
            enigo: runtime.enigo(),
            implementation: MouseImpl::new(runtime).await?,
            pressed_buttons: Mutex::new(Default::default()),
        })
    }

    #[instrument(skip(self), err, ret)]
    pub async fn is_pressed(&self, button: Button) -> Result<bool> {
        self.implementation.is_button_pressed(button).await
    }

    #[instrument(skip(self), err, ret)]
    pub async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent> {
        self.implementation
            .wait_for_button(conditions, cancellation_token)
            .await
    }

    #[instrument(skip(self), err, ret)]
    pub fn scroll(&self, length: i32, axis: Axis) -> Result<()> {
        use enigo::Mouse;

        Ok(self.enigo.lock().unwrap().scroll(length, axis.into())?)
    }

    #[instrument(skip(self), err, ret)]
    pub fn set_position(&self, position: Point, coordinate: Coordinate) -> Result<()> {
        use enigo::Mouse;

        Ok(self
            .enigo
            .lock()
            .unwrap()
            .move_mouse(position.x, position.y, coordinate)?)
    }

    #[instrument(skip(self), err, ret)]
    pub fn position(&self) -> Result<Point> {
        use enigo::Mouse;

        let pos = self.enigo.lock().unwrap().location()?;

        Ok(point(pos.0, pos.1))
    }

    #[instrument(skip(self), err, ret)]
    pub async fn measure_speed(&self, duration: Duration) -> Result<f64> {
        let mut last_position = self.position()?;
        let mut last_time = Instant::now();

        let mut total_distance = 0.0;
        let mut sample_count = 0;

        let start_time = Instant::now();

        while start_time.elapsed() < duration {
            sleep(Duration::from_millis(10)).await;

            let current_position = self.position()?;
            let current_time = Instant::now();

            let delta_time = current_time.duration_since(last_time).as_secs_f32();
            let distance = last_position.distance_to(current_position);

            if delta_time > 0.0 {
                total_distance += distance;
                sample_count += 1;
            }

            last_position = current_position;
            last_time = current_time;
        }

        Ok(if sample_count > 0 {
            total_distance / duration.as_secs_f64()
        } else {
            0.0
        })
    }
}

/// Move options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct MoveOptions {
    /// @default 2000
    pub speed: f64,

    /// @default Tween.SINE_OUT
    pub tween: Tween,

    /// @default 50
    pub perlin_scale: f64,

    /// @default 5
    pub perlin_amplitude: f64,

    /// @default 0
    pub target_randomness: f64,

    /// Interval in milliseconds
    /// @default 10
    pub interval: JsDuration,
}

impl Default for MoveOptions {
    fn default() -> Self {
        Self {
            speed: 2000.,
            tween: Tween::SineOut,
            perlin_scale: 50.,
            perlin_amplitude: 5.,
            target_randomness: 0.,
            interval: Duration::from_millis(10).into(),
        }
    }
}

fn sigmoid(x: f64) -> f64 {
    1. / (1. + (-x).exp())
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub async fn move_(
        &self,
        mut target_position: Point,
        cancellation_token: CancellationToken,
        options: MoveOptions,
        rng: SharedRng,
    ) -> Result<()> {
        if options.target_randomness > 0. {
            target_position =
                Point::random_in_circle(target_position, options.target_randomness, rng.clone())?;
        }

        let start_position = self.position()?;
        let distance = start_position.distance_to(target_position);

        let duration = if options.speed < 0. {
            return Err(MouseError::ParameterError(
                "speed must be greater than zero".into(),
            ));
        } else {
            Duration::from_secs_f64(distance / options.speed)
        };

        if options.interval.0.is_zero() {
            return Err(MouseError::ParameterError("interval cannot be zero".into()));
        }

        let mut perlin_rng = UniformRandomGen::new(rng.next_u32());
        let perlin_seed = rng.next_u32();

        let duration = duration.as_secs_f64();

        if duration < 0. {
            self.set_position(target_position, Coordinate::Abs)?;

            return Ok(());
        }

        let mut tween = FixedTweener::new(
            start_position,
            target_position,
            duration,
            options.tween.into_tween(),
            options.interval.0.as_secs_f64(),
        );

        let (direction_x, direction_y) = (target_position - start_position).normalize();
        let (perpendicular_x, perpendicular_y) = (-direction_y, direction_x);

        while !tween.is_finished() {
            let time = tween.current_time;
            let progress = (time / duration).min(1.0);

            let eased_progress = sigmoid(progress.mul_add(12., -6.));

            let noise_factor = eased_progress * options.perlin_scale;
            let noise = perlin_noise_1d(
                &mut perlin_rng,
                noise_factor.to_f32().unwrap_or_default(),
                perlin_seed,
            )
            .to_f64()
            .unwrap_or_default()
                * options.perlin_amplitude;

            let damping_factor = 1.0 - eased_progress.powi(3); // More easing as it approaches the end

            // Apply perpendicular noise
            let (noise_offset_x, noise_offset_y) = (
                perpendicular_x * noise * damping_factor,
                perpendicular_y * noise * damping_factor,
            );

            let position = tween.move_next()
                + try_point(noise_offset_x, noise_offset_y)
                    .map_err(|err: eyre::Error| MouseError::ParameterError(err.to_string()))?;

            self.set_position(position, Coordinate::Abs)?;

            select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(());
                },
                _ = sleep(options.interval.0) => {},
            }
        }

        self.set_position(target_position, Coordinate::Abs)?;

        Ok(())
    }
}

/// Button click options
/// @extends PressOptions
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ClickOptions {
    /// @skip
    pub press: PressOptions,

    /// @default 1
    pub amount: i32,

    /// @default 0
    pub interval: JsDuration,

    /// @default 0
    pub duration: JsDuration,
}

impl Default for ClickOptions {
    fn default() -> Self {
        Self {
            press: PressOptions::default(),
            amount: 1,
            interval: Duration::ZERO.into(),
            duration: Duration::ZERO.into(),
        }
    }
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub async fn click(&self, options: ClickOptions) -> Result<()> {
        use enigo::Mouse;

        let coordinate = if options.press.relative_position {
            Coordinate::Rel
        } else {
            Coordinate::Abs
        };

        let mut action = {
            if let Some(position) = &options.press.position {
                self.enigo.lock().unwrap().move_mouse(
                    position.inner().x,
                    position.inner().y,
                    coordinate,
                )?;
            }

            let mut enigo = self.enigo.lock().unwrap();

            move |direction| enigo.button(options.press.button.into(), direction)
        };

        for i in 0..options.amount {
            if !options.duration.0.is_zero() {
                let contains = {
                    let lock = self.pressed_buttons.lock().unwrap();
                    lock.contains(&options.press.button)
                };

                if !contains {
                    action(enigo::Direction::Press)?;
                } else {
                    info!(
                        "button {} is already pressed, ignoring",
                        options.press.button
                    );
                }
                sleep(options.duration.0).await;
                action(enigo::Direction::Release)?;
            } else {
                action(enigo::Direction::Click)?;
            }

            info!("removing {} from the pressed buttons", options.press.button);

            {
                let mut lock = self.pressed_buttons.lock().unwrap();
                lock.shift_remove(&options.press.button);
            }

            if !options.interval.0.is_zero() && i + 1 < options.amount {
                sleep(options.interval.0).await;
            }
        }

        Ok(())
    }
}

/// Button double click options
/// @extends ClickOptions
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct DoubleClickOptions {
    /// @skip
    pub click: ClickOptions,

    /// @default 250
    pub delay: JsDuration,
}

impl Default for DoubleClickOptions {
    fn default() -> Self {
        Self {
            click: ClickOptions::default(),
            delay: Duration::from_millis(250).into(),
        }
    }
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub async fn double_click(&self, options: DoubleClickOptions) -> Result<()> {
        self.click(options.click).await?;
        sleep(options.delay.0).await;
        self.click(options.click).await?;

        Ok(())
    }
}

/// Button press options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct PressOptions {
    /// @default Button.LEFT
    pub button: Button,

    /// @default undefined
    pub position: Option<JsPoint>,

    /// @default false
    pub relative_position: bool,
}

impl Default for PressOptions {
    fn default() -> Self {
        Self {
            button: Button::Left,
            position: None,
            relative_position: false,
        }
    }
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub fn press(&self, options: PressOptions) -> Result<()> {
        use enigo::Mouse;

        let contains = {
            let lock = self.pressed_buttons.lock().unwrap();
            lock.contains(&options.button)
        };

        if contains {
            info!("button {} is already pressed, ignoring", options.button);

            return Ok(());
        }

        let coordinate = if options.relative_position {
            Coordinate::Rel
        } else {
            Coordinate::Abs
        };

        if let Some(position) = &options.position {
            self.enigo.lock().unwrap().move_mouse(
                position.inner().x,
                position.inner().y,
                coordinate,
            )?;
        }

        self.enigo
            .lock()
            .unwrap()
            .button(options.button.into(), enigo::Direction::Press)?;

        info!("adding {} to the pressed buttons", options.button);

        {
            let mut lock = self.pressed_buttons.lock().unwrap();
            lock.insert(options.button);
        }

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn release(&self, button: Option<Button>) -> Result<()> {
        use enigo::Mouse;

        let button = if let Some(button) = button {
            let contains = {
                let lock = self.pressed_buttons.lock().unwrap();
                lock.contains(&button)
            };

            if !contains {
                info!("button {button} is not pressed, ignoring");

                return Ok(());
            }

            button
        } else {
            let last_pressed_button = {
                let mut lock = self.pressed_buttons.lock().unwrap();
                lock.pop()
            };

            if let Some(last_pressed_button) = last_pressed_button {
                info!("releasing last pressed button, {last_pressed_button}");

                last_pressed_button
            } else {
                info!("no pressed button, ignoring");

                return Ok(());
            }
        };

        self.enigo
            .lock()
            .unwrap()
            .button(button.into(), Direction::Release)?;

        info!("removing {} from the pressed buttons", button);

        {
            let mut lock = self.pressed_buttons.lock().unwrap();
            lock.shift_remove(&button);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio_util::sync::CancellationToken;
    use tracing_test::traced_test;

    use super::{Mouse, Tween};
    use crate::{
        core::{
            mouse::{ButtonConditions, MoveOptions},
            point::point,
        },
        runtime::{Runtime, shared_rng::SharedRng},
    };

    #[test]
    #[traced_test]
    fn test_position() {
        Runtime::test(|runtime| async {
            let mouse = Arc::new(Mouse::new(runtime).await.unwrap());
            let cancellation_token = CancellationToken::new();
            let rng = SharedRng::default();

            for target in [point(5000, 1000), point(7000, 800), point(4000, 1200)] {
                mouse
                    .move_(
                        target,
                        cancellation_token.clone(),
                        MoveOptions {
                            speed: 2000.,
                            tween: Tween::SineOut,
                            perlin_scale: 50.,
                            perlin_amplitude: 5.,
                            target_randomness: 10.,
                            ..Default::default()
                        },
                        rng.clone(),
                    )
                    .await
                    .unwrap()
            }

            // TODO
            // ...
        });
    }

    #[test]
    #[traced_test]
    fn test_wait_for_button() {
        Runtime::test(async |runtime| {
            let mouse = Arc::new(Mouse::new(runtime).await.unwrap());

            println!("Press any mouse button");
            let button = mouse
                .wait_for_button(
                    ButtonConditions {
                        button: None,
                        direction: None,
                    },
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            println!("Done: {:?}", button);
        });
    }
}
