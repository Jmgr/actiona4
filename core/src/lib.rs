#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::unwrap_used)]
#![deny(unsafe_code)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::future_not_send)]
#![allow(clippy::too_many_arguments)]
#![allow(rustdoc::invalid_html_tags)]

use color_eyre::Result;
use rquickjs::{Coerced, Ctx, Exception, Value};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::error::CommonError;

pub mod api;
pub mod config;
pub mod enigo;
pub mod error;
pub(crate) mod platform;
pub mod platform_info;
pub mod runtime;
pub mod scripting;
pub mod sized_body;
pub mod types;
pub mod updater;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub trait IntoJSError: ToString {
    fn into_js(self, ctx: &Ctx<'_>) -> rquickjs::Error
    where
        Self: Sized,
    {
        Exception::throw_message(ctx, &self.to_string())
    }
}

pub trait IntoJsResult<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> rquickjs::Result<T>;
}

impl<T, E> IntoJsResult<T> for std::result::Result<T, E>
where
    E: IntoJSError,
{
    fn into_js_result(self, ctx: &Ctx<'_>) -> rquickjs::Result<T> {
        self.map_err(|err| err.into_js(ctx))
    }
}

impl<T> IntoJsResult<T> for std::result::Result<T, color_eyre::Report> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> rquickjs::Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

pub trait JsValueToString {
    fn to_string_coerced(&self) -> Result<String>;
}

impl<'js> JsValueToString for Value<'js> {
    fn to_string_coerced(&self) -> Result<String> {
        Ok(self.get::<Coerced<String>>()?.0)
    }
}

#[must_use]
pub fn format_js_value_for_console(value: Value<'_>) -> String {
    let ctx = value.ctx().clone();
    api::console::js::JsConsole::print_value(&ctx, value)
}

#[macro_export]
macro_rules! newtype {
    // --- Internal Rule ---
    // This rule generates the actual struct and impls.
    // It's marked non-exported by convention (starting with `@` or being non-pub).
    (@impls $(#[$attr:meta])* $vis:vis $name:ident, $inner:ty) => {
        $(#[$attr])* // Apply derives passed to this rule
        $vis struct $name($inner); // Assume inner field has same visibility or is private

        impl std::ops::Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
        impl From<$inner> for $name {
            fn from(value: $inner) -> Self { $name(value) }
        }
        impl From<$name> for $inner {
            fn from(value: $name) -> Self { value.0 }
        }
    };

    // --- Public Rules ---

    // Rule 1: Explicit attributes provided (at least one attribute)
    // We use `+` (one or more) to ensure this rule matches only when attributes are present.
    (
        $(#[$attr:meta])+ // Match one or more attributes
        $vis:vis $name:ident, $inner:ty
    ) => {
        // Call the internal rule, passing the captured attributes
        $crate::newtype!(@impls $(#[$attr])* $vis $name, $inner);
    };

    // Rule 2: No attributes provided (use defaults)
    // This rule matches if Rule 1 didn't (because there were zero attributes).
    (
        $vis:vis $name:ident, $inner:ty
    ) => {
        // Call the internal rule, passing the default derives
        $crate::newtype!(@impls
            #[derive(Debug, Clone, Default, PartialEq)] // Default derives
            $vis $name, $inner
        );
    };
}

async fn cancel_on<T, F>(token: &CancellationToken, fut: F) -> color_eyre::Result<T>
where
    F: Future<Output = T>,
{
    select! {
        _ = token.cancelled() => Err(CommonError::Cancelled.into()),
        v = fut => Ok(v),
    }
}
