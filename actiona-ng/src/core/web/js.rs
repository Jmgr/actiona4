use rquickjs::{
    JsLifetime,
    class::{Trace, Tracer},
};

use crate::core::js::classes::SingletonClass;

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Ui")]
pub struct JsWeb {
    inner: super::Web,
}

impl SingletonClass<'_> for JsWeb {}

impl<'js> Trace<'js> for JsWeb {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsWeb {
    /// @skip
    pub async fn new() -> super::Result<Self> {
        Ok(Self {
            inner: super::Web::new(),
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWeb {
    // TODO
}
