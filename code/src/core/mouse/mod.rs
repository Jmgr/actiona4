use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use code_macros::FromJsObject;
use enigo::{Direction, Enigo, InputError, NewConError};
use indexmap::IndexSet;
use noiselib::{perlin::perlin_noise_1d, uniform::UniformRandomGen};
use platform::MouseImplTrait;
use rand::RngCore;
use thiserror::Error;
use tracing::{info, instrument};
use tween::FixedTweener;

use crate::core::point::js::JsPoint;

pub(crate) mod platform;

pub mod js;

pub use enigo::Coordinate;
pub use js::{JsAxis, JsButton, JsTween};
#[cfg(windows)]
use platform::win::MouseImpl;
#[cfg(unix)]
use platform::x11::MouseImpl;

use super::{js::JsDuration, point::Point};
use crate::{core::point::point, runtime::Runtime};

#[derive(Debug, Error)]
pub enum MouseError {
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

impl JsButton {
    fn into_enigo(self) -> enigo::Button {
        use JsButton::*;

        match self {
            Left => enigo::Button::Left,
            Middle => enigo::Button::Middle,
            Right => enigo::Button::Right,
            Back => enigo::Button::Back,
            Forward => enigo::Button::Forward,
        }
    }
}

impl JsAxis {
    fn into_enigo(self) -> enigo::Axis {
        use JsAxis::*;

        match self {
            Horizontal => enigo::Axis::Horizontal,
            Vertical => enigo::Axis::Vertical,
        }
    }
}

impl JsTween {
    fn into_tween<Value: tween::TweenValue>(self) -> Box<dyn tween::Tween<Value>> {
        use JsTween::*;

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

#[derive(Debug)]
pub struct Mouse {
    enigo: Arc<Mutex<Enigo>>,
    implementation: MouseImpl,
    pressed_buttons: IndexSet<JsButton>,
}

// TODO: record
// TODO: drag and drop?

impl Mouse {
    #[instrument]
    pub async fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self {
            enigo: runtime.enigo(),
            implementation: MouseImpl::new(runtime).await?,
            pressed_buttons: Default::default(),
        })
    }

    #[instrument(skip(self), err, ret)]
    pub fn is_pressed(&mut self, button: JsButton) -> Result<bool> {
        // TODO: TEST
        self.implementation.is_button_pressed(button)
    }

    #[instrument(skip(self), err, ret)]
    pub fn scroll(&mut self, length: i32, axis: JsAxis) -> Result<()> {
        use enigo::Mouse;

        Ok(self
            .enigo
            .lock()
            .unwrap()
            .scroll(length, axis.into_enigo())?)
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
    pub fn measure_speed(&self, duration: Duration) -> Result<f32> {
        let mut last_position = self.position()?;
        let mut last_time = Instant::now();

        let mut total_distance = 0.0;
        let mut sample_count = 0;

        let start_time = Instant::now();

        while start_time.elapsed() < duration {
            sleep(Duration::from_millis(10));

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
            total_distance / duration.as_secs_f32()
        } else {
            0.0
        })
    }
}

// IMPORTANT: please update default values below!
/// Move options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct MoveOptions {
    /// @default 2000
    pub speed: f32,

    /// @default Tween.SINE_OUT
    pub tween: JsTween,

    /// @default 50
    pub perlin_scale: f32,

    /// @default 5
    pub perlin_amplitude: f32,

    /// @default 0
    pub target_randomness: f32,

    /// Interval in milliseconds
    /// @default 10
    pub interval: JsDuration,
}

impl Default for MoveOptions {
    fn default() -> Self {
        Self {
            speed: 2000.,
            tween: JsTween::SineOut,
            perlin_scale: 50.,
            perlin_amplitude: 5.,
            target_randomness: 0.,
            interval: JsDuration(Duration::from_millis(10)),
        }
    }
}

fn sigmoid(x: f32) -> f32 {
    1. / (1. + (-x).exp())
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub fn move_(&mut self, mut target_position: Point, options: MoveOptions) -> Result<()> {
        if options.target_randomness > 0. {
            target_position = Point::random_in_circle(target_position, options.target_randomness);
        }

        let start_position = self.position()?;
        let distance = start_position.distance_to(target_position);

        let duration = if options.speed == 0. {
            return Err(MouseError::ParameterError("speed cannot be zero".into()));
        } else {
            Duration::from_secs_f32(distance / options.speed)
        };

        if options.interval.0.is_zero() {
            return Err(MouseError::ParameterError("interval cannot be zero".into()));
        }

        let mut rng = rand::rng();
        let mut perlin_rng = UniformRandomGen::new(rng.next_u32());
        let perlin_seed = rng.next_u32();

        let duration = duration.as_secs_f32();

        if duration < 0. {
            self.set_position(target_position, Coordinate::Abs)?;

            return Ok(());
        }

        let mut tween = FixedTweener::new(
            start_position,
            target_position,
            duration,
            options.tween.into_tween(),
            options.interval.0.as_secs_f32(),
        );

        let direction = (target_position - start_position).normalize();
        let perpendicular = point(-direction.y, direction.x);

        while !tween.is_finished() {
            let time = tween.current_time;
            let progress = (time / duration).min(1.0);

            let eased_progress = sigmoid(progress * 12. - 6.);

            let noise_factor = eased_progress * options.perlin_scale;
            let noise = perlin_noise_1d(&mut perlin_rng, noise_factor, perlin_seed)
                * options.perlin_amplitude;

            let damping_factor = 1.0 - eased_progress.powi(3); // More easing as it approaches the end
            let noise_offset = perpendicular.scale(noise * damping_factor); // Apply perpendicular noise

            let position = tween.move_next() + noise_offset;

            self.set_position(position, Coordinate::Abs)?;

            sleep(options.interval.0);
        }

        self.set_position(target_position, Coordinate::Abs)?;

        Ok(())
    }
}

// IMPORTANT: please update default values below!
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
            interval: JsDuration(Duration::ZERO),
            duration: JsDuration(Duration::ZERO),
        }
    }
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub fn click(&mut self, options: ClickOptions) -> Result<()> {
        use enigo::Mouse;

        let coordinate = if options.press.relative_position {
            Coordinate::Rel
        } else {
            Coordinate::Abs
        };

        let mut action = {
            let mut enigo = self.enigo.lock().unwrap();

            if let Some(position) = &options.press.position {
                enigo.move_mouse(position.inner().x, position.inner().y, coordinate)?;
            }

            move |direction| enigo.button(options.press.button.into_enigo(), direction)
        };

        for i in 0..options.amount {
            if !options.duration.0.is_zero() {
                if !self.pressed_buttons.contains(&options.press.button) {
                    action(enigo::Direction::Press)?;
                } else {
                    info!(
                        "button {} is already pressed, ignoring",
                        options.press.button
                    );
                }
                sleep(options.duration.0);
                action(enigo::Direction::Release)?;
            } else {
                action(enigo::Direction::Click)?;
            }

            info!("removing {} from the pressed buttons", options.press.button);
            self.pressed_buttons.shift_remove(&options.press.button);

            if !options.interval.0.is_zero() && i + 1 < options.amount {
                sleep(options.interval.0);
            }
        }

        Ok(())
    }
}

// IMPORTANT: please update default values below!
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
            delay: JsDuration(Duration::from_millis(250)),
        }
    }
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub fn double_click(&mut self, options: DoubleClickOptions) -> Result<()> {
        self.click(options.click)?;
        sleep(options.delay.0);
        self.click(options.click)?;

        Ok(())
    }
}

// IMPORTANT: please update default values below!
/// Button press options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct PressOptions {
    /// @default Button.LEFT
    pub button: JsButton,

    /// @default undefined
    pub position: Option<JsPoint>,

    /// @default false
    pub relative_position: bool,
}

impl Default for PressOptions {
    fn default() -> Self {
        Self {
            button: JsButton::Left,
            position: None,
            relative_position: false,
        }
    }
}

impl Mouse {
    #[instrument(skip(self), err, ret)]
    pub fn press(&mut self, options: PressOptions) -> Result<()> {
        use enigo::Mouse;

        if self.pressed_buttons.contains(&options.button) {
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
            .button(options.button.into_enigo(), enigo::Direction::Press)?;

        info!("adding {} to the pressed buttons", options.button);
        self.pressed_buttons.insert(options.button);

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn release(&mut self, button: Option<JsButton>) -> Result<()> {
        use enigo::Mouse;

        let button = if let Some(button) = button {
            if !self.pressed_buttons.contains(&button) {
                info!("button {button} is not pressed, ignoring");

                return Ok(());
            }

            button
        } else if let Some(last_pressed_button) = self.pressed_buttons.pop() {
            info!("releasing last pressed button, {last_pressed_button}");

            last_pressed_button
        } else {
            info!("no pressed button, ignoring");

            return Ok(());
        };

        self.enigo
            .lock()
            .unwrap()
            .button(button.into_enigo(), Direction::Release)?;

        info!("removing {} from the pressed buttons", button);
        self.pressed_buttons.shift_remove(&button);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::{JsTween, Mouse};
    use crate::{
        core::{mouse::MoveOptions, point::point},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    fn test_position() {
        Runtime::test(async |runtime| {
            let mut mouse = Mouse::new(runtime).await.unwrap();

            for target in [point(5000, 1000), point(7000, 800), point(4000, 1200)] {
                mouse
                    .move_(
                        target,
                        MoveOptions {
                            speed: 2000.,
                            tween: JsTween::SineOut,
                            perlin_scale: 50.,
                            perlin_amplitude: 5.,
                            target_randomness: 10.,
                            ..Default::default()
                        },
                    )
                    .unwrap()
            }

            // TODO
            // ...
        });
    }
}
