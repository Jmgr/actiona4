use std::time::SystemTime;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Weekday};
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
        date::system_time_from_date,
        duration::JsDuration,
        task::task_with_token,
    },
    cancel_on,
};

/// Day of the week, used with `datetime.waitForDayOfWeek`.
///
/// ```ts
/// // Wait until next Monday at midnight
/// await datetime.waitForDayOfWeek(DayOfWeek.Monday);
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

/// Provides time-condition based waiting.
///
/// All `waitFor*` methods return a cancellable `Task` that resolves at the
/// next occurrence of the specified time condition.
///
/// ```ts
/// // Wait until next 13:00:00
/// await datetime.waitForHour(13);
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
        let target = system_time_from_date(ctx.clone(), date)?;
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = target.duration_since(SystemTime::now()).unwrap_or_default();
            cancel_on(&token, tokio::time::sleep(duration))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until the next occurrence of the given hour (0–23) at minute 0, second 0.
    ///
    /// Always waits for the *next* occurrence: if the current time is already past
    /// `hour:00:00` today, it waits until tomorrow.
    ///
    /// ```ts
    /// // Run something every day at 09:00
    /// while (true) {
    ///   await datetime.waitForHour(9);
    ///   doMorningTask();
    /// }
    /// ```
    ///
    /// ```ts
    /// // With cancellation support
    /// const controller = new AbortController();
    /// await datetime.waitForHour(13, { signal: controller.signal });
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_for_hour<'js>(
        &self,
        ctx: Ctx<'js>,
        hour: u32,
        options: Opt<JsWaitOptions>,
    ) -> Result<Promise<'js>> {
        if hour > 23 {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "hour must be between 0 and 23",
            ));
        }
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = next_occurrence_of_hour(hour)
                .duration_since(SystemTime::now())
                .unwrap_or_default();
            cancel_on(&token, tokio::time::sleep(duration))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until the next occurrence of the given minute (0–59), at second 0.
    ///
    /// Always waits for the *next* occurrence: if the current minute is already
    /// past `minute:00`, it waits until the same minute in the next hour.
    ///
    /// ```ts
    /// // Run something every hour at HH:30:00
    /// while (true) {
    ///   await datetime.waitForMinute(30);
    ///   doHalfHourTask();
    /// }
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_for_minute<'js>(
        &self,
        ctx: Ctx<'js>,
        minute: u32,
        options: Opt<JsWaitOptions>,
    ) -> Result<Promise<'js>> {
        if minute > 59 {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "minute must be between 0 and 59",
            ));
        }
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = next_occurrence_of_minute(minute)
                .duration_since(SystemTime::now())
                .unwrap_or_default();
            cancel_on(&token, tokio::time::sleep(duration))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until the next occurrence of the given day of the month (1–31) at midnight.
    ///
    /// Always waits for the *next* occurrence: if the current day of month is
    /// already past (or equal to) `day`, it waits until that day in the next month.
    /// Months that are shorter than `day` are skipped automatically.
    ///
    /// ```ts
    /// // Run something on the 1st of every month
    /// while (true) {
    ///   await datetime.waitForDayOfMonth(1);
    ///   doMonthlyTask();
    /// }
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_for_day_of_month<'js>(
        &self,
        ctx: Ctx<'js>,
        day: u32,
        options: Opt<JsWaitOptions>,
    ) -> Result<Promise<'js>> {
        if !(1..=31).contains(&day) {
            return Err(rquickjs::Exception::throw_range(
                &ctx,
                "day must be between 1 and 31",
            ));
        }
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = next_occurrence_of_day_of_month(day)
                .duration_since(SystemTime::now())
                .unwrap_or_default();
            cancel_on(&token, tokio::time::sleep(duration))
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until the next occurrence of the given weekday at midnight.
    ///
    /// Always waits for the *next* occurrence: if today is already that weekday,
    /// it waits until the same weekday next week.
    ///
    /// ```ts
    /// // Run something every Monday
    /// while (true) {
    ///   await datetime.waitForDayOfWeek(DayOfWeek.Monday);
    ///   doWeeklyTask();
    /// }
    /// ```
    ///
    /// ```ts
    /// // With cancellation
    /// const controller = new AbortController();
    /// await datetime.waitForDayOfWeek(DayOfWeek.Friday, { signal: controller.signal });
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_for_day_of_week<'js>(
        &self,
        ctx: Ctx<'js>,
        day: JsDayOfWeek,
        options: Opt<JsWaitOptions>,
    ) -> Result<Promise<'js>> {
        let weekday = Weekday::from(day);
        let signal = options.0.and_then(|o| o.signal);
        task_with_token(ctx, signal, async move |ctx, token| {
            let duration = next_occurrence_of_day_of_week(weekday)
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

/// Returns the next `SystemTime` at which the clock will read `hour:00:00` local time.
/// Always returns a future time (never the current moment).
fn next_occurrence_of_hour(hour: u32) -> SystemTime {
    let now = Local::now();
    let naive = now
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(hour, 0, 0).expect("validated hour should be in range"));
    let candidate = Local.from_local_datetime(&naive).unwrap();
    if candidate > now {
        SystemTime::from(candidate)
    } else {
        SystemTime::from(candidate + Duration::days(1))
    }
}

/// Returns the next `SystemTime` at which the clock will read `HH:minute:00` local time.
/// Always returns a future time.
fn next_occurrence_of_minute(minute: u32) -> SystemTime {
    let now = Local::now();
    let naive = now.date_naive().and_time(
        NaiveTime::from_hms_opt(now.hour(), minute, 0)
            .expect("validated minute should be in range"),
    );
    let candidate = Local.from_local_datetime(&naive).unwrap();
    if candidate > now {
        SystemTime::from(candidate)
    } else {
        SystemTime::from(candidate + Duration::hours(1))
    }
}

/// Returns the next `SystemTime` at which the local date will be `day` (day-of-month), at midnight.
/// Skips months that do not have that day (e.g. day 31 in April). Always returns a future time.
fn next_occurrence_of_day_of_month(day: u32) -> SystemTime {
    let now = Local::now();

    // Try current month first.
    if let Some(st) = date_at_midnight(now.year(), now.month(), day)
        && st > SystemTime::now()
    {
        return st;
    }

    // Walk forward month by month until we find a valid date.
    let mut year = now.year();
    let mut month = now.month() + 1;
    loop {
        if month > 12 {
            month = 1;
            year += 1;
        }
        if let Some(st) = date_at_midnight(year, month, day) {
            return st;
        }
        month += 1;
    }
}

/// Returns the next `SystemTime` at midnight on the given `weekday` in local time.
/// Always returns a future time (never today).
fn next_occurrence_of_day_of_week(weekday: Weekday) -> SystemTime {
    let now = Local::now();
    let today = now.weekday();
    let days_from_mon_target = i64::from(weekday.num_days_from_monday());
    let days_from_mon_today = i64::from(today.num_days_from_monday());
    let days_until = (days_from_mon_target - days_from_mon_today).rem_euclid(7);
    // Always wait for the *next* occurrence, never today.
    let days_until = if days_until == 0 { 7 } else { days_until };

    let target_date = now.date_naive() + Duration::days(days_until);
    date_at_midnight_naive(target_date)
}

fn date_at_midnight(year: i32, month: u32, day: u32) -> Option<SystemTime> {
    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|naive| SystemTime::from(Local.from_local_datetime(&naive).unwrap()))
}

fn date_at_midnight_naive(date: chrono::NaiveDate) -> SystemTime {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .expect("midnight should always be a valid time");
    SystemTime::from(Local.from_local_datetime(&naive).unwrap())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

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
            // A date in the past should resolve without delay.
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
    fn test_wait_for_hour_invalid() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine.eval::<()>("datetime.waitForHour(24)").await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_wait_for_minute_invalid() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine.eval::<()>("datetime.waitForMinute(60)").await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_wait_for_day_of_month_invalid() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval::<()>("datetime.waitForDayOfMonth(0)")
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

    #[test]
    fn test_wait_for_day_of_week_can_be_cancelled() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<()>(
                    r#"
                    const task = datetime.waitForDayOfWeek(DayOfWeek.Monday);
                    task.cancel();
                    await task;
                    "#,
                )
                .await;
            assert_eq!(result.unwrap_err().to_string(), "Error: Cancelled");
        });
    }

    #[test]
    fn test_next_occurrence_of_hour_is_future() {
        let now = SystemTime::now();
        for hour in 0..24 {
            let next = super::next_occurrence_of_hour(hour);
            assert!(
                next > now,
                "hour {hour}: next occurrence should be in the future"
            );
        }
    }

    #[test]
    fn test_next_occurrence_of_minute_is_future() {
        let now = SystemTime::now();
        for minute in 0..60 {
            let next = super::next_occurrence_of_minute(minute);
            assert!(
                next > now,
                "minute {minute}: next occurrence should be in the future"
            );
        }
    }

    #[test]
    fn test_next_occurrence_of_day_of_month_is_future() {
        let now = SystemTime::now();
        for day in 1..=31 {
            let next = super::next_occurrence_of_day_of_month(day);
            assert!(
                next > now,
                "day {day}: next occurrence should be in the future"
            );
        }
    }

    #[test]
    fn test_next_occurrence_of_day_of_week_is_future() {
        use chrono::Weekday;
        let now = SystemTime::now();
        for weekday in [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ] {
            let next = super::next_occurrence_of_day_of_week(weekday);
            assert!(
                next > now,
                "{weekday}: next occurrence should be in the future"
            );
        }
    }

    #[test]
    fn test_next_occurrence_of_day_of_month_within_one_month() {
        use std::time::Duration;
        // The next occurrence of any day is at most ~62 days away (e.g. day 31 in Feb → skip to March 31).
        let now = SystemTime::now();
        for day in 1..=31_u32 {
            let next = super::next_occurrence_of_day_of_month(day);
            let diff = next.duration_since(now).unwrap();
            assert!(
                diff <= Duration::from_secs(63 * 24 * 3600),
                "day {day}: next occurrence is unexpectedly far: {diff:?}"
            );
        }
    }
}
