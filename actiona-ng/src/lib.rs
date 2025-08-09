//#![recursion_limit = "1024"] // TODO: is this still needed?
//#![warn(clippy::all, clippy::dbg_macro, clippy::float_cmp_const, clippy::get_unwrap, clippy::mem_forget, clippy::nursery, clippy::pedantic)]
#![warn(clippy::all, clippy::nursery)]

use rquickjs::Ctx;
pub use slotmap::{
    Key as SlotmapKey, KeyData as SlotmapKeyData, SecondaryMap as SlotmapSecondaryMap,
};

pub mod core;
pub mod enigo;
pub mod error;
pub(crate) mod platform;
pub mod runtime;
pub mod scripting;

pub trait IntoJS<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> rquickjs::Result<T>;
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
