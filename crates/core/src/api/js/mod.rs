pub mod abort_controller;
pub mod classes;
pub mod concurrency;
pub mod date;
pub mod duration;
pub mod event_handle;
pub mod global;
pub mod task;

use rquickjs::{Class, Ctx, FromJs, Object, Result, Value, class::JsClass};

pub trait FromJsField<'js>: Sized {
    fn from_js_field(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self>;
}

impl<'js, T> FromJsField<'js> for T
where
    T: FromJs<'js>,
{
    fn from_js_field(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        T::from_js(ctx, value)
    }
}

/// Returns whether `object` uses the prototype registered for the Rust-backed JS class `T`.
///
/// We check prototype equality instead of calling `object.instance_of::<T>()` directly because
/// rquickjs implements that via `JS_GetOpaque2`, which throws on plain JS objects. Several of our
/// overload parsers intentionally accept either a real Rust class instance or a plain object
/// literal, so they need a non-throwing way to recognize the registered-class case first.
///
/// @skip
pub fn has_registered_class_prototype<'js, T>(ctx: &Ctx<'js>, object: &Object<'js>) -> Result<bool>
where
    T: JsClass<'js>,
{
    let Some(object_prototype) = object.get_prototype() else {
        return Ok(false);
    };
    let Some(class_prototype) = Class::<T>::prototype(ctx)? else {
        return Ok(false);
    };

    Ok(object_prototype == class_prototype)
}
