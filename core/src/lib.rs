//#![warn(clippy::all, clippy::dbg_macro, clippy::float_cmp_const, clippy::get_unwrap, clippy::mem_forget, clippy::nursery, clippy::pedantic)]
#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
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

use crate::error::{CommonError, Error};

pub mod api;
pub mod config;
pub mod enigo;
pub mod error;
pub(crate) mod platform;
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

pub trait JsValueToString {
    fn to_string_coerced(&self) -> Result<String>;
}

impl<'js> JsValueToString for Value<'js> {
    fn to_string_coerced(&self) -> Result<String> {
        Ok(self.get::<Coerced<String>>()?.0)
    }
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

async fn cancel_on<T, F>(token: &CancellationToken, fut: F) -> Result<T, Error>
where
    F: Future<Output = T>,
{
    select! {
        _ = token.cancelled() => Err(Error::CommonError(CommonError::Cancelled)),
        v = fut => Ok(v),
    }
}

// TODO: check all errors
// TODO: check all token cancellation return a Cancelled error
// TODO: check all unwraps
// TODO: check if we can use cancel_on in a few places where select! is used
// TODO: display a tray icon, enabled by default when waitAtEnd is true
// TODO: enigo::set_dpi_awareness()
/*
Note that the top-left hand corner of the desktop is not necessarily the same as the screen.
If the user uses a desktop with multiple monitors, the top-left hand corner of the desktop is
the top-left hand corner of the main monitor on Windows and macOS or the top-left of the
leftmost monitor on X11.
*/
/*
use windows_sys::Win32::Globalization::CP_UTF8;
use windows_sys::Win32::System::Console::SetConsoleOutputCP;

unsafe {
    SetConsoleOutputCP(CP_UTF8);
}
*/
/*
You are running actiona4-run version 0.1.0, latest version is 1.0.1,
released 3d ago.
*/
// TODO: 3d ago? Oo
// TODO: Maybe remove Arc<Foo> and make Foo clonable directly
