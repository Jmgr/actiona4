//! Proc macros used by Actiona's Rust <-> JS bridge.
//!
//! Each macro below includes a usage example.

use proc_macro::TokenStream;

mod class;
mod consts;
mod default_args;
mod from_js_object;
mod js_enum;
mod methods;
mod options;
mod platform;
mod platform_validate;
mod serde;

// TODO: use https://github.com/rquickjs/rquickjs-serde

/// Derives `rquickjs::FromJs` for option-like structs with named fields.
///
/// Missing JS properties keep the struct's Rust default.
/// Property names are read using the active rename rules.
///
/// # Example
/// ```rust,ignore
/// use macros::FromJsObject;
///
/// #[derive(Debug, Default, FromJsObject)]
/// struct OpenOptions {
///     read: bool,
///     write: bool,
///     create_new: bool,
/// }
/// ```
#[proc_macro_derive(FromJsObject)]
pub fn derive_from_js_object(input: TokenStream) -> TokenStream {
    from_js_object::derive(input)
}

/// Expands field/variant attributes into rustdoc instructions consumed by the
/// doc-generator (`@default`, `@platforms`) and generates defaults metadata.
///
/// Use this on option-like structs with public fields. It will:
/// - inject `@options` so the doc-generator treats it as an options object
/// - emit `@default` for public fields
/// - generate `Default` for the options struct
///
/// # Example
/// ```rust,ignore
/// use macros::{options, PlatformValidate};
///
/// #[options]
/// #[derive(PlatformValidate)]
/// struct DemoOptions {
///     #[default]
///     enabled: bool,
///     #[platform(only = "linux")]
///     linux_only: Option<String>,
/// }
/// ```
#[proc_macro_attribute]
pub fn options(arguments: TokenStream, item: TokenStream) -> TokenStream {
    options::expand(arguments, item, "options", true)
}

/// Expands enum variants with `@platforms` rustdoc instructions and, when the
/// type name starts with `Js`, injects `#[serde(rename = "...")]` and `@rename`
/// by stripping the prefix.
///
/// # Example
/// ```rust,ignore
/// use macros::{js_enum, PlatformValidate};
///
/// #[derive(PlatformValidate)]
/// #[js_enum]
/// pub enum JsClipboardMode {
///     Clipboard,
///     #[platform(only = "linux")]
///     Selection,
/// }
/// ```
#[proc_macro_attribute]
pub fn js_enum(arguments: TokenStream, item: TokenStream) -> TokenStream {
    js_enum::expand(arguments, item)
}

/// Adds a runtime platform guard to a function returning `Result<...>`.
///
/// Supported keys:
/// - `only = "linux" | "windows" | "wayland" | "x11"`
/// - `not = "linux" | "windows" | "wayland" | "x11"`
///
/// If a `Ctx` parameter is present, the current platform is read from runtime
/// user-data. Otherwise it falls back to `Platform::detect()`.
///
/// Also injects a rustdoc `@platforms ...` line if none exists, so the
/// doc-generator can emit `@platform`.
///
/// # Example
/// ```rust,ignore
/// use macros::platform;
/// use rquickjs::{Ctx, Result};
///
/// #[platform(only = "linux")]
/// fn send_signal(ctx: Ctx<'_>, pid: u32) -> Result<()> {
///     let _ = (ctx, pid);
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn platform(arguments: TokenStream, item: TokenStream) -> TokenStream {
    platform::expand(arguments, item)
}

/// Derives `validate_for_platform(platform)` for structs and enums.
///
/// Annotate platform-limited fields or variants with `#[platform(...)]`.
///
/// # Example
/// ```rust,ignore
/// use macros::PlatformValidate;
///
/// #[derive(PlatformValidate)]
/// enum ClipboardMode {
///     Clipboard,
///     #[platform(only = "linux")]
///     Selection,
/// }
/// ```
#[proc_macro_derive(PlatformValidate, attributes(platform))]
pub fn derive_platform_validate(input: TokenStream) -> TokenStream {
    platform_validate::derive(input)
}

/// Wraps `#[rquickjs::class]` and auto-fills `rename` by removing `Js` from
/// the Rust type name (for example `JsMouse` -> `"Mouse"`).
#[proc_macro_attribute]
pub fn js_class(arguments: TokenStream, item: TokenStream) -> TokenStream {
    class::expand(arguments, item)
}

/// Wraps `#[rquickjs::methods]` and processes helper attributes like `#[get]`/`#[set]`.
///
/// Defaults to `rename_all` when not specified.
#[proc_macro_attribute]
pub fn js_methods(arguments: TokenStream, item: TokenStream) -> TokenStream {
    methods::expand(arguments, item)
}

/// Derives `rquickjs::IntoJs` through `serde`.
///
/// # Example
/// ```rust,ignore
/// use macros::IntoSerde;
/// use serde::Serialize;
///
/// #[derive(Serialize, IntoSerde)]
/// struct Payload {
///     value: String,
/// }
/// ```
#[proc_macro_derive(IntoSerde)]
pub fn into_serde(input: TokenStream) -> TokenStream {
    serde::derive_into_serde(input)
}

/// Derives `rquickjs::FromJs` through `serde`.
///
/// # Example
/// ```rust,ignore
/// use macros::FromSerde;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, FromSerde)]
/// struct Payload {
///     value: String,
/// }
/// ```
#[proc_macro_derive(FromSerde)]
pub fn from_serde(input: TokenStream) -> TokenStream {
    serde::derive_from_serde(input)
}
