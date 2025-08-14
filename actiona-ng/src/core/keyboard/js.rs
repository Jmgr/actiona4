use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
};

use crate::{IntoJS, core::js::classes::SingletonClass, runtime::Runtime};

impl<T> IntoJS<T> for super::Result<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for super::Keyboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

pub type JsKey = super::Key;

/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Keyboard")]
pub struct JsKeyboard {
    inner: super::Keyboard,
}

impl SingletonClass<'_> for JsKeyboard {}

impl JsKeyboard {
    /// @skip
    pub fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        Ok(Self {
            inner: super::Keyboard::new(runtime)?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsKeyboard {
    // TODO
}
