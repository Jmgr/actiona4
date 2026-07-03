//! Proc macros used by Actiona's Rust <-> JS bridge.
//!
//! Each macro below includes a usage example.

use proc_macro::TokenStream;

mod action_definition;
mod class;
mod consts;
mod default_args;
mod editor;
mod from_js_object;
mod js_enum;
mod methods;
mod options;
mod platform;
mod platform_validate;
mod rpc_protocol;
mod serde;

/// Derives `rquickjs::FromJs` for option-like structs with named fields.
///
/// - Fields typed `Option<T>`: absent JS key → `None`; present key → `Some(value)`.
/// - Non-`Option` fields with scalar types (`bool`, integers, floats, `Vec`): absent JS key →
///   struct's `Default` value (e.g. `false`, `0`, `[]`).
/// - Non-`Option` fields with other types (`String`, custom structs/enums, etc.): absent JS key →
///   JS `Error` ("missing required field '...'").
///
/// Property names are converted to camelCase.
///
/// # Example
/// ```rust,ignore
/// use macros::FromJsObject;
///
/// #[derive(Debug, Default, FromJsObject)]
/// struct OpenOptions {
///     /// Required: must always be supplied from JS.
///     path: String,
///     /// Optional: defaults to `false` when absent (bool inferred as optional).
///     write: bool,
/// }
/// ```
#[proc_macro_derive(FromJsObject, attributes(default))]
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

/// Wraps `#[rquickjs::methods]` and processes helper attributes like
/// `#[get]`, `#[set]`, and `#[prop]`.
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

/// Generates an extension IPC protocol from an async trait declaration.
///
/// The trait is schema input only; it is replaced with:
/// - a protocol marker struct
/// - request/response enums for each used direction
/// - a `Protocol` impl
/// - side implementation traits named `<Protocol>Host` and `<Protocol>Extension`
/// - typed host-call methods on `Host<Protocol>`
/// - typed extension-call methods on `Extension<Protocol>`
///
/// Methods marked with `#[host_call]` are host-to-extension calls. Methods
/// marked with `#[extension_call]` are extension-to-host calls. Request and
/// response variants are derived from the method name.
///
/// # Example
/// ```rust,ignore
/// use macros::rpc_protocol;
///
/// #[rpc_protocol]
/// pub trait SelectionProtocol {
///     #[host_call]
///     async fn select_rect() -> Option<Rect>;
///
///     #[extension_call]
///     async fn current_window_title() -> Option<String>;
/// }
/// ```
#[proc_macro_attribute]
pub fn rpc_protocol(arguments: TokenStream, item: TokenStream) -> TokenStream {
    rpc_protocol::expand(arguments, item)
}

/// Derives `ActionDefinition` and `WithDefinition` for an action struct.
///
/// `#[action(icon = ...)]` sets the icon; `id` defaults to the struct name in
/// snake_case (override with `#[action(id = "...")]`). Fields tagged
/// `#[parameter]` become the action's `Parameter` list, in declaration order,
/// typed through that field's `ParameterStorage` impl. Per-field settings can
/// be overridden with `#[parameter(key = value, ...)]`, where each `key`
/// assigns into the field type's `ParameterStorage::Settings`.
///
/// Name and description translation keys are derived from the id
/// (`action-<id-kebab>`) and, per parameter, from the field name
/// (`action-<id-kebab>-<field-kebab>`).
///
/// # Example
/// ```rust,ignore
/// use macros::Action;
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Default, Action)]
/// #[action(icon = MousePointerClick)]
/// pub struct Click {
///     #[parameter]
///     pub position: Scriptable<Point>,
/// }
/// ```
#[proc_macro_derive(Action, attributes(action, parameter))]
pub fn derive_action(input: TokenStream) -> TokenStream {
    action_definition::action::derive(input)
}

/// Generates the `ACTION_DEFINITIONS` slice from the `ActionInstance` enum,
/// keeping the enum the single source of truth. Each variant must be a newtype
/// over its action struct (`Click(Click)`); the derive lists that struct's
/// `DEFINITION` const, so the slice stays a zero-cost `const`.
///
/// # Example
/// ```rust,ignore
/// use macros::ActionDefinitions;
///
/// #[derive(ActionDefinitions)]
/// pub enum ActionInstance {
///     Click(Click),
///     MessageBox(MessageBox),
/// }
/// // expands to:
/// // pub const ACTION_DEFINITIONS: &[ActionDefinition] =
/// //     &[<Click>::DEFINITION, <MessageBox>::DEFINITION];
/// ```
#[proc_macro_derive(ActionDefinitions)]
pub fn derive_action_definitions(input: TokenStream) -> TokenStream {
    action_definition::definitions::derive(input)
}

/// Derives `ParameterStorage` for a unit-style enum used as an action
/// parameter, backing it with `EnumParameter`.
///
/// Requires `#[serde(rename_all = "kebab-case")]`; the derive reuses each
/// variant's serialized name as its `EnumParameterVariant::id`, so
/// `#[serde(rename = "...")]` on a variant also renames its metadata id. The
/// variant's translation key is `enum-<EnumName-kebab>.<variant-id>`.
///
/// # Example
/// ```rust,ignore
/// use macros::ActionEnum;
///
/// #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, ActionEnum)]
/// #[serde(rename_all = "kebab-case")]
/// pub enum MessageBoxButtons {
///     #[default]
///     Ok,
///     OkCancel,
/// }
/// ```
#[proc_macro_derive(ActionEnum)]
pub fn derive_action_enum(input: TokenStream) -> TokenStream {
    action_definition::action_enum::derive(input)
}

/// Derives `into_kind(self) -> ParameterKind` for a `<Name>Parameter` struct,
/// wrapping it in the `ParameterKind` variant matching `Name`.
///
/// With `#[parameter(storage = T)]`, also derives `impl ParameterStorage for
/// T`, using `Self::DEFAULT` (from `const_default::ConstDefault`) as
/// `DEFAULT_SETTINGS`. Without it, only `into_kind` is generated — used for
/// `EnumParameter`, whose `ParameterStorage` impl is instead generated
/// per-enum by [`macro@ActionEnum`].
///
/// # Example
/// ```rust,ignore
/// use macros::Parameter;
///
/// #[derive(Debug, ConstDefault, Parameter)]
/// #[parameter(storage = Scriptable<String>)]
/// pub struct TextParameter {
///     pub max_length: Option<u64>,
/// }
/// ```
#[proc_macro_derive(Parameter, attributes(parameter))]
pub fn derive_parameter(input: TokenStream) -> TokenStream {
    action_definition::parameter::derive(input)
}

/// Apply to a trait of `async fn` methods. Because the trait lives in a crate
/// both the host and the wasm UI depend on, the generated client and server
/// share the exact same method names and argument/return types — the call is
/// checked at compile time on *both* ends, which a JS frontend (Tauri) cannot
/// do without a codegen step.
///
/// ```ignore
/// #[rpc]
/// pub trait Api {
///     async fn add_ten(&self, value: i32) -> i32;
/// }
/// ```
///
/// Generates, in the same module:
/// * `trait Api` — the host implements it (`impl Api for HostApi { async fn … }`).
/// * `ApiClient<T: Transport>` — the UI calls `client.add_ten(5).await`, fully typed.
/// * `api_serve(api, cmd, json) -> Option<Future<json>>` — host dispatch by name.
/// * `__rpc_<method>` modules holding the by-name argument struct + the name.
///
/// The surrounding module must have `Transport` and `RpcError` in scope (both
/// are hand-written in `common::rpc`, where the trait is defined).
#[proc_macro_attribute]
pub fn rpc(attr: TokenStream, item: TokenStream) -> TokenStream {
    editor::rpc::expand(attr, item)
}
