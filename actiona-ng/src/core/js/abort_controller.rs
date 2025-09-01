use rquickjs::{Ctx, JsLifetime, class::Trace};
use tokio_util::sync::CancellationToken;

use crate::{core::js::classes::ValueClass, runtime::WithUserData};

#[derive(Debug, Clone, JsLifetime)]
#[rquickjs::class]
pub struct JsAbortSignal {
    token: CancellationToken,
}

impl<'js> ValueClass<'js> for JsAbortSignal {}

impl<'js> Trace<'js> for JsAbortSignal {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

// TODO: fix

#[rquickjs::methods]
impl JsAbortSignal {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Self {
        Self {
            token: ctx.user_data().cancellation_token().child_token(),
        }
    }
}

impl JsAbortSignal {
    /// @skip
    pub fn into_token(self) -> CancellationToken {
        self.token
    }
}

pub trait IntoToken {
    fn into_token(self) -> Option<CancellationToken>;
}

impl IntoToken for Option<JsAbortSignal> {
    fn into_token(self) -> Option<CancellationToken> {
        self.map(|token| token.into_token())
    }
}

impl IntoToken for CancellationToken {
    fn into_token(self) -> Option<CancellationToken> {
        Some(self)
    }
}

/// @prop readonly signal: AbortSignal
#[derive(Debug, JsLifetime)]
#[rquickjs::class]
pub struct JsAbortController {
    token: CancellationToken,
}

impl<'js> ValueClass<'js> for JsAbortController {}

impl<'js> Trace<'js> for JsAbortController {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl JsAbortController {
    /// @constructor
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Self {
        Self {
            token: ctx.user_data().cancellation_token().child_token(),
        }
    }

    pub fn abort(&self) {
        self.token.cancel();
    }

    /// @skip
    #[qjs(get)]
    pub fn signal(&self) -> JsAbortSignal {
        JsAbortSignal {
            token: self.token.child_token(),
        }
    }
}
