//#![recursion_limit = "1024"] // TODO: is this still needed?

use eyre::{Result, eyre};
use rquickjs::{Context as JsContext, Ctx, FromJs};
pub use slotmap::{
    Key as SlotmapKey, KeyData as SlotmapKeyData, SecondaryMap as SlotmapSecondaryMap,
};

pub mod core;
pub mod enigo;
pub(crate) mod platform;
pub mod runtime;
pub mod ts_to_js;

pub trait IntoJS<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> rquickjs::Result<T>;
}

pub fn eval<T>(js_context: &JsContext, source: &str) -> Result<T>
where
    for<'any_js> T: FromJs<'any_js> + Send,
{
    js_context.with(|ctx| {
        // TODO: use .catch
        ctx.eval::<T, _>(source).map_err(|_| {
            let e = ctx.catch();
            eprintln!("err {:?}", e); // TMP
            eyre!(
                "{}",
                e.as_exception()
                    .expect("caught value should be an exception")
                    .message()
                    .expect("exception should have a message")
            )
        })
    })
}

#[macro_export]
macro_rules! newtype {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, Default, PartialEq)]
        pub struct $name($inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}
