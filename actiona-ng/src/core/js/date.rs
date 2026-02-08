use std::time::{Duration, SystemTime, UNIX_EPOCH};

use color_eyre::eyre::eyre;
use rquickjs::{
    Ctx, Exception, Function, Object, Result,
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
    let date_object: Object = ctx.globals().get(PredefinedAtom::Date)?;
    if !date.is_instance_of(&date_object) {
        return Err(Exception::throw_message(
            &ctx,
            &format!("Expected a Date parameter, got {}", date.type_name()),
        ));
    }

    let get_time: Function = date.get("getTime")?;
    let mut args = Args::new(ctx, 0);
    args.this(date)?;
    let time: u64 = get_time.call_arg(args)?;

    Ok(UNIX_EPOCH + Duration::from_millis(time))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{
        core::js::date::{date_from_system_time, system_time_from_date},
        runtime::Runtime,
    };

    #[test]
    fn test_date_system_time() {
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
        })
    }
}
