#![allow(clippy::needless_pass_by_value)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use color_eyre::eyre::eyre;
use jiff::civil;
use rquickjs::{
    Ctx, Exception, FromJs, Function, IntoJs, Object, Result, Value,
    atom::PredefinedAtom,
    function::{Args, Constructor},
};

use crate::IntoJsResult;

/// Converts a `SystemTime` to a JavaScript `Date` object.
/// @skip
pub fn date_from_system_time<'js>(ctx: &Ctx<'js>, system_time: &SystemTime) -> Result<Object<'js>> {
    let global = ctx.globals();
    let date_constructor: Constructor = global.get("Date")?;

    let duration = system_time.duration_since(UNIX_EPOCH).into_js_result(ctx)?;
    let millis = u64::try_from(duration.as_millis())
        .map_err(|err| eyre!("{err}"))
        .into_js_result(ctx)?;

    date_constructor.construct::<_, Object<'js>>((millis,))
}

/// Converts a JavaScript `Date` object to a `SystemTime`.
/// @skip
pub fn system_time_from_date<'js>(ctx: Ctx<'js>, date: Object<'js>) -> Result<SystemTime> {
    check_is_date(&ctx, &date)?;

    let get_time: Function = date.get("getTime")?;
    let mut args = Args::new(ctx, 0);
    args.this(date)?;
    let time: u64 = get_time.call_arg(args)?;

    Ok(UNIX_EPOCH + Duration::from_millis(time))
}

fn check_is_date<'js>(ctx: &Ctx<'js>, date: &Object<'js>) -> Result<()> {
    let date_object: Object = ctx.globals().get(PredefinedAtom::Date)?;
    if !date.is_instance_of(&date_object) {
        return Err(Exception::throw_message(
            ctx,
            &format!("Expected a Date parameter, got {}", date.type_name()),
        ));
    }
    Ok(())
}

fn call_date_getter<'js>(ctx: &Ctx<'js>, date: &Object<'js>, getter: &str) -> Result<i32> {
    let getter: Function = date.get(getter)?;
    let mut args = Args::new(ctx.clone(), 0);
    args.this(date.clone())?;
    getter.call_arg(args)
}

/// A calendar day, converted from and to a JavaScript `Date` in local time.
///
/// The time of day of a `Date` converted to this type is ignored, and a `Date` converted from it
/// is at local midnight.
/// @skip
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JsDate(pub civil::Date);

impl<'js> FromJs<'js> for JsDate {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        let Some(date) = value.as_object() else {
            return Err(Exception::throw_message(
                ctx,
                &format!("Expected a Date parameter, got {}", value.type_name()),
            ));
        };
        check_is_date(ctx, date)?;

        let year = call_date_getter(ctx, date, "getFullYear")?;
        let month = call_date_getter(ctx, date, "getMonth")? + 1;
        let day = call_date_getter(ctx, date, "getDate")?;

        let date = i16::try_from(year)
            .map_err(|err| eyre!("{err}"))
            .and_then(|year| {
                civil::Date::new(
                    year,
                    i8::try_from(month).map_err(|err| eyre!("{err}"))?,
                    i8::try_from(day).map_err(|err| eyre!("{err}"))?,
                )
                .map_err(|err| eyre!("{err}"))
            })
            .into_js_result(ctx)?;

        Ok(Self(date))
    }
}

impl<'js> IntoJs<'js> for JsDate {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        let date_constructor: Constructor = ctx.globals().get(PredefinedAtom::Date)?;
        let date: Object<'js> = date_constructor.construct((
            i32::from(self.0.year()),
            i32::from(self.0.month()) - 1,
            i32::from(self.0.day()),
        ))?;
        Ok(date.into_value())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use jiff::civil::date;

    use crate::{
        api::js::date::{JsDate, date_from_system_time, system_time_from_date},
        runtime::Runtime,
    };

    #[test]
    fn date_system_time() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .with::<_, _>(|ctx| {
                    let time = SystemTime::now();
                    let date = date_from_system_time(&ctx, &time)?;
                    let time2 = system_time_from_date(ctx, date)?;

                    let to_ms = |t: SystemTime| t.duration_since(UNIX_EPOCH).unwrap().as_millis();

                    assert_eq!(to_ms(time), to_ms(time2));
                    Ok(())
                })
                .await
                .unwrap();
        });
    }

    #[test]
    fn civil_date() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .with::<_, _>(|ctx| {
                    let leap_day = date(2024, 2, 29);
                    ctx.globals().set("leapDay", JsDate(leap_day))?;
                    let parts: Vec<i32> =
                        ctx.eval("[leapDay.getFullYear(), leapDay.getMonth(), leapDay.getDate(), leapDay.getHours()]")?;
                    assert_eq!(parts, vec![2024, 1, 29, 0]);

                    let from_js: JsDate = ctx.eval("new Date(1999, 11, 31, 23, 59)")?;
                    assert_eq!(from_js.0, date(1999, 12, 31));

                    assert!(ctx.eval::<JsDate, _>("\"2024-02-29\"").is_err());
                    Ok(())
                })
                .await
                .unwrap();
        });
    }
}
