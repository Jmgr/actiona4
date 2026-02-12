use std::time::Duration;

use humantime::parse_duration;
use rquickjs::{Ctx, Exception, FromJs, Result, Value};

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
        Ok(Self(if let Some(value) = value.as_string() {
            let value = value.to_string()?;

            parse_duration(&value).map_err(|err| {
                Exception::throw_message(ctx, &format!("Failed to parse duration '{value}': {err}"))
            })?
        } else {
            let secs = f64::from_js(ctx, value)?;

            Duration::from_secs_f64(secs)
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{api::js::duration::JsDuration, runtime::Runtime};

    #[test]
    fn from_js_duration_float() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let duration = script_engine.eval_async::<JsDuration>("1.5").await.unwrap();
            assert!((duration.0.as_secs_f64() - 1.5).abs() < 1e-9);
        });
    }

    #[test]
    fn from_js_duration_int() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let duration = script_engine.eval_async::<JsDuration>("2").await.unwrap();
            assert_eq!(duration.0, Duration::from_secs(2));
        });
    }

    #[test]
    fn from_js_duration_humantime() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let duration = script_engine
                .eval_async::<JsDuration>("\"10m\"")
                .await
                .unwrap();
            assert_eq!(duration.0, Duration::from_secs(10 * 60));
        });
    }

    #[test]
    fn from_js_duration_invalid_suffix() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine.eval_async::<JsDuration>("\"10q\"").await;
            assert!(result.is_err());
        });
    }
}
