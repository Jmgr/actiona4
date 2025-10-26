use std::time::{Duration, SystemTime, UNIX_EPOCH};

use eyre::eyre;
use rquickjs::{
    Ctx, Exception, Function, Object, Result,
    atom::PredefinedAtom,
    function::{Args, Constructor},
};

use crate::IntoJsResult;

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
