use std::sync::Arc;

use itertools::Itertools;
use macros::FromJsObject;
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};

use crate::{
    IntoJsResult,
    core::{
        js::classes::{HostClass, register_host_class},
        system::hardware::{Component, Hardware, Motherboard},
    },
};

/// Hardware information.
///
/// ```ts
/// const hw = system.hardware;
/// const board = hw.motherboard;
/// const components = await hw.listComponents();
///
/// console.log(hw.vendorName, board.name, components.length);
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Hardware")]
pub struct JsHardware {
    inner: Arc<Hardware>,
    motherboard: Arc<Motherboard>,
}

impl<'js> HostClass<'js> for JsHardware {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsMotherboard>(ctx)?;
        register_host_class::<JsComponent>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsHardware {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsHardware {
    /// @skip
    #[must_use]
    pub fn new(inner: Arc<Hardware>) -> Self {
        let motherboard = Arc::new(inner.motherboard().clone());

        Self { inner, motherboard }
    }
}

/// List components options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListComponentsOptions {
    /// Rescan
    /// @default `true`
    pub rescan: bool,
}

impl Default for ListComponentsOptions {
    fn default() -> Self {
        Self { rescan: true }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsHardware {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// Family
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.inner.family()
    }

    /// Serial number
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.inner.serial_number()
    }

    /// Stock keeping unit
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn stock_keeping_unit(&self) -> Option<&str> {
        self.inner.stock_keeping_unit()
    }

    /// Version
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.inner.version()
    }

    /// Uuid
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn uuid(&self) -> Option<&str> {
        self.inner.uuid()
    }

    /// Vendor name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.inner.vendor_name()
    }

    /// Motherboard
    /// @get
    /// @readonly
    #[qjs(get)]
    #[must_use]
    pub fn motherboard(&self) -> JsMotherboard {
        self.motherboard.clone().into()
    }

    /// Hardware components
    /// @readonly
    pub async fn list_components<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<ListComponentsOptions>,
    ) -> Result<Vec<JsComponent>> {
        let options = options.unwrap_or_default();
        let components = self
            .inner
            .refresh_components(options.rescan)
            .await
            .into_js_result(&ctx)?;

        Ok(components.into_iter().map(JsComponent::from).collect_vec())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// Motherboard details.
///
/// ```ts
/// const board = system.hardware.motherboard;
/// console.log(board.vendorName, board.name, board.version);
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Motherboard")]
pub struct JsMotherboard {
    inner: Arc<Motherboard>,
}

impl<'js> HostClass<'js> for JsMotherboard {}

impl<'js> Trace<'js> for JsMotherboard {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Arc<Motherboard>> for JsMotherboard {
    fn from(value: Arc<Motherboard>) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMotherboard {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// Vendor name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.inner.vendor_name()
    }

    /// Version
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.inner.version()
    }

    /// Serial number
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.inner.serial_number()
    }

    /// Asset tag
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn asset_tag(&self) -> Option<&str> {
        self.inner.asset_tag()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// A hardware component (for example a thermal sensor).
///
/// ```ts
/// const components = await system.hardware.listComponents();
/// const component = components[0];
/// if (component) {
///   console.log(component.label, component.temperature);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Component")]
pub struct JsComponent {
    inner: Component,
}

impl<'js> HostClass<'js> for JsComponent {}

impl<'js> Trace<'js> for JsComponent {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Component> for JsComponent {
    fn from(value: Component) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsComponent {
    /// Label
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn label(&self) -> &str {
        self.inner.label()
    }

    /// ID
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.inner.id().as_deref()
    }

    /// Temperature
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn temperature(&self) -> Option<f64> {
        self.inner.temperature().as_deref().copied()
    }

    /// Maximum temperature
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn max_temperature(&self) -> Option<f64> {
        self.inner.max_temperature().as_deref().copied()
    }

    /// Critical temperature
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn critical_temperature(&self) -> Option<f64> {
        self.inner.critical_temperature().as_deref().copied()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}
