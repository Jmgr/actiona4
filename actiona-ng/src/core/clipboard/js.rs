use std::fmt::Debug;

use eyre::eyre;
use rquickjs::{
    JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};

use crate::core::SingletonClass;
use crate::{IntoJS, newtype};

newtype!(
    #[derive(JsLifetime)]
    Clipboard,
    arboard::Clipboard
);

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Clipboard").finish()
    }
}

/// @global
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Clipboard")]
pub struct JsClipboard {
    inner: Clipboard,
}

impl<'js> Trace<'js> for JsClipboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl SingletonClass<'_> for JsClipboard {}

impl JsClipboard {
    /// @skip
    pub fn new<'js>(ctx: &Ctx<'js>) -> Result<Self> {
        Ok(Self {
            inner: Clipboard(
                arboard::Clipboard::new()
                    .map_err(|err| eyre!("{err}"))
                    .into_js(&ctx)?,
            ),
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboard {
    pub fn set_text<'js>(&mut self, ctx: Ctx<'js>, text: String) -> Result<()> {
        self.inner
            .set_text(text)
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)?;

        Ok(())
    }
}
