use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use macros::{js_class, js_methods};
use rquickjs::{
    JsLifetime,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
};

use crate::api::js::classes::HostClass;
/// Unique identifier for a registered event handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HandleId(u64);

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

impl Default for HandleId {
    fn default() -> Self {
        Self(NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A registry that can remove a specific handle by ID.
pub(crate) trait HandleRegistry: std::fmt::Debug + Send + Sync {
    fn remove_handle(&self, id: HandleId);
}

/// A handle to a registered event listener. Call `.cancel()` to unregister it.
///
/// ```ts
/// const handle = keyboard.onText("btw", "by the way");
/// // ... later:
/// handle.cancel();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsEventHandle {
    id: HandleId,
    registry: Arc<dyn HandleRegistry>,
}

impl<'js> Trace<'js> for JsEventHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl HostClass<'_> for JsEventHandle {}

impl JsEventHandle {
    pub(crate) fn new(id: HandleId, registry: Arc<dyn HandleRegistry>) -> Self {
        Self { id, registry }
    }
}

#[js_methods]
impl JsEventHandle {
    /// Unregisters this event listener.
    pub fn cancel(&self) {
        self.registry.remove_handle(self.id);
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "EventHandle".to_string()
    }
}
