pub mod abort_controller;
pub mod classes;
pub mod concurrency;
pub mod date;
pub mod duration;
pub mod event_handle;
pub mod global;
pub mod task;

use rquickjs::{Ctx, FromJs, Result, Value};

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
