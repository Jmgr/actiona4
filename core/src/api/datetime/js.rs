use std::time::SystemTime;

use chrono::{Datelike, Duration, Local, NaiveTime, TimeZone, Weekday};
use derive_more::Display;
use macros::{FromJsObject, FromSerde, IntoSerde, js_class, js_enum, js_methods, options};
use rquickjs::{
    Ctx, JsLifetime, Object, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    IntoJsResult,
    api::js::{
        abort_controller::JsAbortSignal,
        classes::{SingletonClass, register_enum},
        duration::JsDuration,
        task::task_with_token,
    },
    cancel_on,
};

/// Day of the week, used with `datetime.waitForSchedule`.
///
/// ```ts
/// // Wait until next Monday at 09:00
/// await datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday, hour: 9 });
/// ```
///
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    Hash,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[js_enum]
pub enum JsDayOfWeek {
    /// `DayOfWeek.Monday`
    Monday,
    /// `DayOfWeek.Tuesday`
    Tuesday,
    /// `DayOfWeek.Wednesday`
    Wednesday,
    /// `DayOfWeek.Thursday`
    Thursday,
    /// `DayOfWeek.Friday`
    Friday,
    /// `DayOfWeek.Saturday`
    Saturday,
    /// `DayOfWeek.Sunday`
    Sunday,
}

impl From<JsDayOfWeek> for Weekday {
    fn from(day: JsDayOfWeek) -> Self {
        match day {
            JsDayOfWeek::Monday => Self::Mon,
            JsDayOfWeek::Tuesday => Self::Tue,
            JsDayOfWeek::Wednesday => Self::Wed,
            JsDayOfWeek::Thursday => Self::Thu,
            JsDayOfWeek::Friday => Self::Fri,
            JsDayOfWeek::Saturday => Self::Sat,
            JsDayOfWeek::Sunday => Self::Sun,
        }
    }
}

/// Options for datetime wait methods.
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsWaitOptions {
    /// Abort signal to cancel the wait.
    pub signal: Option<JsAbortSignal>,
}

/// Schedule options for `datetime.waitForSchedule`.
///
/// All fields are optional. Missing day fields (`dayOfWeek`, `dayOfMonth`) match
/// any day. Missing time fields (`hour`, `minute`, `second`) default to `0`.
///
/// The method always waits for the **next strictly future** occurrence that
/// satisfies all specified constraints.
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsScheduleOptions {
    /// Target hour (0–23). Defaults to `0`.
    pub hour: Option<u32>,
    /// Target minute (0–59). Defaults to `0`.
    pub minute: Option<u32>,
    /// Target second (0–59). Defaults to `0`.
    pub second: Option<u32>,
    /// Target weekday. Matches any weekday if omitted.
    pub day_of_week: Option<JsDayOfWeek>,
    /// Target day of the month (1–31). Matches any day if omitted.
    /// Months shorter than `dayOfMonth` are skipped automatically.
    pub day_of_month: Option<u32>,
    /// Abort signal to cancel the wait.
    pub signal: Option<JsAbortSignal>,
}

/// Provides time-condition based waiting.
///
/// All `waitFor*` methods return a cancellable `Task` that resolves at the
/// next occurrence of the specified time condition.
///
/// ```ts
/// // Wait until next 13:15
/// await datetime.waitForSchedule({ hour: 13, minute: 15 });
/// ```
///
/// ```ts
/// // Wait until next Monday at 09:30
/// await datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday, hour: 9, minute: 30 });
/// ```
///
/// ```ts
/// // Wait for a duration (alias for sleep)
/// await datetime.waitFor("2s");
/// ```
///
/// ```ts
/// // Wait until a specific date
/// await datetime.waitUntil(new Date("2026-12-31T23:59:59"));
/// ```
///
/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[js_class]
pub struct JsDatetime {}

impl<'js> Trace<'js> for JsDatetime {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsDatetime {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsDayOfWeek>(ctx)
    }
}

#[js_methods]
impl JsDatetime {
    /// @skip
    #[qjs(constructor)]
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Waits for the given duration. Alias for `sleep`.
    ///
    /// ```ts
    /// await datetime.waitFor(500);     // 500 ms
    /// await datetime.waitFor("1s");    // 1 second
    /// await datetime.waitFor("30m");   // 30 minutes
    /// ```
    ///
    /// Numeric values are interpreted as milliseconds.
    /// @returns Task<void>
    pub fn wait_for<'js>(
        &self,
        ctx: Ctx<'js>,
        duration: JsDuration,
        options: Opt<JsWaitOptions>,
    ) -> Result<Promise<'js>> {
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            cancel_on(&token, tokio::time::sleep(duration.0))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until a specific `Date` is reached.
    ///
    /// Resolves immediately if the date is in the past.
    ///
    /// ```ts
    /// await datetime.waitUntil(new Date("2026-12-31T23:59:59"));
    /// ```
    ///
    /// ```ts
    /// // Wait until 1 second from now
    /// const target = new Date(Date.now() + 1000);
    /// await datetime.waitUntil(target);
    /// ```
    ///
    /// @param date: Date
    /// @returns Task<void>
    pub fn wait_until<'js>(
        &self,
        ctx: Ctx<'js>,
        date: Object<'js>,
        options: Opt<JsWaitOptions>,
    ) -> Result<Promise<'js>> {
        let target = crate::api::js::date::system_time_from_date(ctx.clone(), date)?;
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = target.duration_since(SystemTime::now()).unwrap_or_default();
            cancel_on(&token, tokio::time::sleep(duration))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until the next occurrence matching the given schedule.
    ///
    /// Missing day fields (`dayOfWeek`, `dayOfMonth`) match any day.
    /// Missing time fields (`hour`, `minute`) default to `0`.
    /// Always waits for the *next strictly future* occurrence.
    ///
    /// ```ts
    /// // Wait until next 13:15 (any day)
    /// await datetime.waitForSchedule({ hour: 13, minute: 15 });
    /// ```
    ///
    /// ```ts
    /// // Wait until next Monday at 09:30
    /// await datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday, hour: 9, minute: 30 });
    /// ```
    ///
    /// ```ts
    /// // Wait until the next :15 of any hour
    /// await datetime.waitForSchedule({ minute: 15 });
    /// ```
    ///
    /// ```ts
    /// // Wait until the 1st of every month at midnight
    /// while (true) {
    ///   await datetime.waitForSchedule({ dayOfMonth: 1 });
    ///   doMonthlyTask();
    /// }
    /// ```
    ///
    /// ```ts
    /// // With cancellation
    /// const controller = new AbortController();
    /// await datetime.waitForSchedule({ hour: 9, signal: controller.signal });
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_for_schedule<'js>(
        &self,
        ctx: Ctx<'js>,
        options: JsScheduleOptions,
    ) -> Result<Promise<'js>> {
        if options.hour.is_some_and(|h| h > 23) {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "hour must be between 0 and 23",
            ));
        }
        if options.minute.is_some_and(|m| m > 59) {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "minute must be between 0 and 59",
            ));
        }
        if options.second.is_some_and(|s| s > 59) {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "second must be between 0 and 59",
            ));
        }
        if options.day_of_month.is_some_and(|d| !(1..=31).contains(&d)) {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "dayOfMonth must be between 1 and 31",
            ));
        }

        let signal = options.signal;
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = next_occurrence_of_schedule(
                options.hour,
                options.minute,
                options.second,
                options.day_of_week,
                options.day_of_month,
            )
            .duration_since(SystemTime::now())
            .unwrap_or_default();
            cancel_on(&token, tokio::time::sleep(duration))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Returns a string representation of the `datetime` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Datetime".to_string()
    }
}

/// Returns the next `SystemTime` that satisfies all non-`None` schedule constraints.
///
/// - `hour` / `minute` / `second`: target time of day; absent fields default to `0`.
/// - `day_of_week` / `day_of_month`: day constraints; absent means any day.
///
/// Always returns a strictly future time.
fn next_occurrence_of_schedule(
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
    day_of_week: Option<JsDayOfWeek>,
    day_of_month: Option<u32>,
) -> SystemTime {
    let target_time =
        NaiveTime::from_hms_opt(hour.unwrap_or(0), minute.unwrap_or(0), second.unwrap_or(0))
            .expect("hour, minute and second already validated");
    let now = Local::now();
    let mut candidate = now.date_naive();

    // If today's slot is not strictly in the future, start searching from tomorrow.
    let today_dt = Local
        .from_local_datetime(&candidate.and_time(target_time))
        .unwrap();
    if today_dt <= now {
        candidate += Duration::days(1);
    }

    loop {
        let dow_ok = day_of_week.is_none_or(|dow| candidate.weekday() == Weekday::from(dow));
        let dom_ok = day_of_month.is_none_or(|dom| candidate.day() == dom);

        if dow_ok && dom_ok {
            let naive = candidate.and_time(target_time);
            return SystemTime::from(Local.from_local_datetime(&naive).unwrap());
        }

        candidate += Duration::days(1);
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use chrono::{Datelike, Local, Timelike};

    use super::JsDayOfWeek;
    use crate::runtime::Runtime;

    #[test]
    fn test_wait_for() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let start = SystemTime::now();
            script_engine
                .eval_async::<()>("await datetime.waitFor(\"100ms\")")
                .await
                .unwrap();
            assert!(start.elapsed().unwrap() >= Duration::from_millis(100));
        });
    }

    #[test]
    fn test_wait_until_past_date_resolves_immediately() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>("await datetime.waitUntil(new Date(0))")
                .await
                .unwrap();
        });
    }

    #[test]
    fn test_wait_until_near_future() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let start = SystemTime::now();
            script_engine
                .eval_async::<()>("await datetime.waitUntil(new Date(Date.now() + 100))")
                .await
                .unwrap();
            assert!(start.elapsed().unwrap() >= Duration::from_millis(100));
        });
    }

    #[test]
    fn test_wait_for_can_be_cancelled() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<()>(
                    r#"
                    const task = datetime.waitFor("10s");
                    task.cancel();
                    await task;
                    "#,
                )
                .await;
            assert_eq!(result.unwrap_err().to_string(), "Error: Cancelled");
        });
    }

    #[test]
    fn test_wait_until_can_be_cancelled() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<()>(
                    r#"
                    const task = datetime.waitUntil(new Date(Date.now() + 10000));
                    task.cancel();
                    await task;
                    "#,
                )
                .await;
            assert_eq!(result.unwrap_err().to_string(), "Error: Cancelled");
        });
    }

    #[test]
    fn test_wait_for_schedule_can_be_cancelled() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<()>(
                    r#"
                    const task = datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday });
                    task.cancel();
                    await task;
                    "#,
                )
                .await;
            assert_eq!(result.unwrap_err().to_string(), "Error: Cancelled");
        });
    }

    #[test]
    fn test_wait_for_schedule_invalid_hour() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval::<()>("datetime.waitForSchedule({ hour: 24 })")
                .await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_wait_for_schedule_invalid_minute() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval::<()>("datetime.waitForSchedule({ minute: 60 })")
                .await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_wait_for_schedule_invalid_day_of_month() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval::<()>("datetime.waitForSchedule({ dayOfMonth: 0 })")
                .await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_day_of_week_enum_accessible() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let value = script_engine
                .eval::<String>("DayOfWeek.Monday")
                .await
                .unwrap();
            assert_eq!(value, "Monday");
        });
    }

    // ── Unit tests for next_occurrence_of_schedule ────────────────────────────

    #[test]
    fn test_next_occurrence_is_always_future() {
        let now = SystemTime::now();
        for hour in [0u32, 9, 13, 23] {
            for minute in [0u32, 15, 30, 59] {
                let next =
                    super::next_occurrence_of_schedule(Some(hour), Some(minute), None, None, None);
                assert!(
                    next > now,
                    "h={hour} m={minute}: next occurrence should be in the future"
                );
            }
        }
        // No constraints: next midnight
        assert!(super::next_occurrence_of_schedule(None, None, None, None, None) > now);
    }

    #[test]
    fn test_next_occurrence_respects_day_of_week() {
        use chrono::Weekday;
        for dow in [
            JsDayOfWeek::Monday,
            JsDayOfWeek::Tuesday,
            JsDayOfWeek::Wednesday,
            JsDayOfWeek::Thursday,
            JsDayOfWeek::Friday,
            JsDayOfWeek::Saturday,
            JsDayOfWeek::Sunday,
        ] {
            let next = super::next_occurrence_of_schedule(None, None, None, Some(dow), None);
            let next_local = chrono::DateTime::<Local>::from(next);
            let expected = Weekday::from(dow);
            assert_eq!(
                next_local.weekday(),
                expected,
                "{dow}: weekday should match"
            );
        }
    }

    #[test]
    fn test_next_occurrence_respects_day_of_month() {
        for dom in [1u32, 15, 28] {
            let next = super::next_occurrence_of_schedule(None, None, None, None, Some(dom));
            let next_local = chrono::DateTime::<Local>::from(next);
            assert_eq!(next_local.day(), dom, "day of month should match");
        }
    }

    #[test]
    fn test_next_occurrence_respects_hour_minute_second() {
        let next = super::next_occurrence_of_schedule(Some(13), Some(15), Some(30), None, None);
        let next_local = chrono::DateTime::<Local>::from(next);
        assert_eq!(next_local.hour(), 13);
        assert_eq!(next_local.minute(), 15);
        assert_eq!(next_local.second(), 30);
    }

    #[test]
    fn test_next_occurrence_day_of_week_within_one_week() {
        let now = SystemTime::now();
        for dow in [
            JsDayOfWeek::Monday,
            JsDayOfWeek::Tuesday,
            JsDayOfWeek::Wednesday,
            JsDayOfWeek::Thursday,
            JsDayOfWeek::Friday,
            JsDayOfWeek::Saturday,
            JsDayOfWeek::Sunday,
        ] {
            let next = super::next_occurrence_of_schedule(None, None, None, Some(dow), None);
            let diff = next.duration_since(now).unwrap();
            assert!(
                diff <= Duration::from_secs(7 * 24 * 3600),
                "{dow}: next occurrence is unexpectedly far: {diff:?}"
            );
        }
    }

    #[test]
    fn test_next_occurrence_day_of_month_within_two_months() {
        let now = SystemTime::now();
        for dom in 1..=31u32 {
            let next = super::next_occurrence_of_schedule(None, None, None, None, Some(dom));
            let diff = next.duration_since(now).unwrap();
            assert!(
                diff <= Duration::from_secs(63 * 24 * 3600),
                "day {dom}: next occurrence is unexpectedly far: {diff:?}"
            );
        }
    }
}
