use std::time::Duration;

use rquickjs::{Ctx, FromJs, Result, Value};

#[derive(Clone, Copy, Debug)]
pub struct JsDuration(pub Duration);

impl From<Duration> for JsDuration {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<JsDuration> for Duration {
    fn from(duration: JsDuration) -> Self {
        duration.0
    }
}

impl<'js> FromJs<'js> for JsDuration {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let ms = f64::from_js(ctx, value)?;
        Ok(Self(ms_to_duration(ms)))
    }
}

pub fn ms_to_duration(ms: f64) -> Duration {
    Duration::from_secs_f64(ms / 1000_f64)
}
