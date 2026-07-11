use std::time::Duration;

use action_definition::{
    actions::{Wait, wait::WaitUnit},
    post_run::PostRun,
    scriptable::Scriptable,
};
use tokio::{select, time::sleep};

use crate::{
    ExecutionContext, PreparedWait, ResolveParam, ResolveParamError, RunError, RunErrorKind,
    Runnable, Waitable, run_prepared_wait,
};

fn parse_wait_unit(value: &str) -> Option<WaitUnit> {
    serde_plain::from_str(value.trim()).ok()
}

async fn resolve_unit(
    unit: &Scriptable<WaitUnit>,
    parameter: &'static str,
    context: &ExecutionContext,
) -> Result<WaitUnit, RunError> {
    match unit {
        Scriptable::Static { value } => Ok(*value),
        Scriptable::Script { source } => {
            let value = context
                .script_engine
                .eval_async::<String>(source)
                .await
                .map_err(|err| RunError::new(ResolveParamError::new(parameter, err)))?;
            parse_wait_unit(&value)
                .ok_or_else(|| RunError::new(RunErrorKind::InvalidWaitUnit { value }))
        }
    }
}

fn to_duration(duration: f64, unit: WaitUnit) -> Result<Duration, RunErrorKind> {
    if !duration.is_finite() || duration < 0.0 {
        return Err(RunErrorKind::InvalidWaitDuration);
    }

    let seconds = match unit {
        WaitUnit::Milliseconds => duration / 1_000.0,
        WaitUnit::Seconds => duration,
        WaitUnit::Minutes => duration * 60.0,
        WaitUnit::Hours => duration * 60.0 * 60.0,
        WaitUnit::Days => duration * 24.0 * 60.0 * 60.0,
    };

    if !seconds.is_finite() {
        return Err(RunErrorKind::InvalidWaitDuration);
    }

    Duration::try_from_secs_f64(seconds).map_err(|_| RunErrorKind::InvalidWaitDuration)
}

impl Waitable for Wait {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        let duration = self.duration.resolve(context).await?;
        let unit = resolve_unit(self.unit.value(), self.unit.name(), context).await?;
        let duration = to_duration(duration, unit).map_err(RunError::new)?;

        Ok(PreparedWait::new(move |token| async move {
            select! {
                _ = token.cancelled() => Err(RunError::new(RunErrorKind::Canceled)),
                _ = sleep(duration) => Ok(()),
            }
        }))
    }
}

impl Runnable for Wait {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        run_prepared_wait(self, context).await?;
        Ok(PostRun::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{WaitUnit, parse_wait_unit, to_duration};

    #[test]
    fn parses_wait_units_from_script_strings() {
        assert!(matches!(
            parse_wait_unit("milliseconds"),
            Some(WaitUnit::Milliseconds)
        ));
        assert!(matches!(
            parse_wait_unit("seconds"),
            Some(WaitUnit::Seconds)
        ));
        assert!(matches!(
            parse_wait_unit("minutes"),
            Some(WaitUnit::Minutes)
        ));
        assert!(matches!(parse_wait_unit("hours"), Some(WaitUnit::Hours)));
        assert!(matches!(parse_wait_unit("days"), Some(WaitUnit::Days)));
        assert!(parse_wait_unit("sec").is_none());
        assert!(parse_wait_unit("fortnights").is_none());
    }

    #[test]
    fn converts_wait_duration_to_std_duration() {
        assert_eq!(
            to_duration(1.5, WaitUnit::Seconds).unwrap(),
            std::time::Duration::from_millis(1500)
        );
        assert_eq!(
            to_duration(2.0, WaitUnit::Minutes).unwrap(),
            std::time::Duration::from_secs(120)
        );
        assert!(to_duration(-1.0, WaitUnit::Seconds).is_err());
        assert!(to_duration(f64::NAN, WaitUnit::Seconds).is_err());
        assert!(to_duration(f64::MAX, WaitUnit::Days).is_err());
    }
}
