//! Shared string constants used across the macro crate.

// ── Rustdoc instruction tokens consumed by the doc-generator ────────────────
/// Marks a struct as an options object in the generated docs.
pub const INSTR_OPTIONS: &str = "@options";
/// Marks a field as having a documented default value.
pub const INSTR_DEFAULT: &str = "@default";
/// Excludes a field from the generated docs and default inference.
pub const INSTR_SKIP: &str = "@skip";
/// Marks an item with platform availability information.
pub const INSTR_PLATFORMS: &str = "@platforms";
/// Marks a method as a getter in the generated docs.
pub const INSTR_GET: &str = "@get";
/// Marks an enum's JS-facing name (after stripping the `Js` prefix).
pub const INSTR_RENAME: &str = "@rename";

// ── rquickjs rename convention ───────────────────────────────────────────────
/// Default `rename_all` value injected by `#[js_methods]`.
pub const RENAME_ALL_CAMEL_CASE: &str = "camelCase";

// ── Naming conventions ───────────────────────────────────────────────────────
/// Prefix stripped from Rust type names to derive the JS class name.
pub const JS_TYPE_PREFIX: &str = "Js";
/// Prefix on raw identifiers (`r#keyword`) stripped before generating labels.
pub const RAW_IDENT_PREFIX: &str = "r#";

// ── Platform identifiers (lowercase, used in `#[platform(only = "...")]`) ───
pub const PLATFORM_LINUX: &str = "linux";
pub const PLATFORM_WINDOWS: &str = "windows";
pub const PLATFORM_WAYLAND: &str = "wayland";
pub const PLATFORM_X11: &str = "x11";

// ── Platform display names (used in error messages) ──────────────────────────
pub const PLATFORM_DISPLAY_LINUX: &str = "Linux";
pub const PLATFORM_DISPLAY_WINDOWS: &str = "Windows";
pub const PLATFORM_DISPLAY_WAYLAND: &str = "Wayland";
pub const PLATFORM_DISPLAY_X11: &str = "X11";
