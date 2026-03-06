use itertools::Itertools;
use macros::{FromJsObject, js_class, js_methods, options};
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};

use crate::{
    IntoJsResult,
    api::{
        js::classes::{HostClass, register_host_class},
        system::hardware::{Hardware, Motherboard, TemperatureSensor},
    },
    types::display::display_with_type,
};

/// Hardware information.
///
/// ```ts
/// const hw = system.hardware;
/// const board = hw.motherboard;
/// const temperatureSensors = await hw.listTemperatureSensors();
///
/// println(hw.vendorName, board.name, temperatureSensors.length);
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsHardware {
    inner: Hardware,
    motherboard: Motherboard,
}

impl<'js> HostClass<'js> for JsHardware {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsMotherboard>(ctx)?;
        register_host_class::<JsTemperatureSensor>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsHardware {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsHardware {
    /// @skip
    #[must_use]
    pub fn new(inner: Hardware) -> Self {
        let motherboard = inner.motherboard().clone();

        Self { inner, motherboard }
    }
}

/// List temperature sensors options
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListTemperatureSensorsOptions {
    /// Rescan
    #[default(true)]
    pub rescan: bool,
}

#[js_methods]
impl JsHardware {
    /// Name
    #[get]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// Family
    #[get]
    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.inner.family()
    }

    /// Serial number
    #[get]
    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.inner.serial_number()
    }

    /// Stock keeping unit
    #[get]
    #[must_use]
    pub fn stock_keeping_unit(&self) -> Option<&str> {
        self.inner.stock_keeping_unit()
    }

    /// Version
    #[get]
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.inner.version()
    }

    /// Uuid
    #[get]
    #[must_use]
    pub fn uuid(&self) -> Option<&str> {
        self.inner.uuid()
    }

    /// Vendor name
    #[get]
    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.inner.vendor_name()
    }

    /// Motherboard
    /// @readonly
    #[get]
    #[must_use]
    pub fn motherboard(&self) -> JsMotherboard {
        self.motherboard.clone().into()
    }

    /// Hardware temperature sensors
    /// @readonly
    pub async fn list_temperature_sensors<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<ListTemperatureSensorsOptions>,
    ) -> Result<Vec<JsTemperatureSensor>> {
        let options = options.unwrap_or_default();
        let temperature_sensors = self
            .inner
            .refresh_temperature_sensors(options.rescan)
            .await
            .into_js_result(&ctx)?;

        Ok(temperature_sensors
            .into_iter()
            .map(JsTemperatureSensor::from)
            .collect_vec())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Hardware", &self.inner)
    }
}

/// Motherboard details.
///
/// ```ts
/// const board = system.hardware.motherboard;
/// println(board.vendorName, board.name, board.version);
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsMotherboard {
    inner: Motherboard,
}

impl<'js> HostClass<'js> for JsMotherboard {}

impl<'js> Trace<'js> for JsMotherboard {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Motherboard> for JsMotherboard {
    fn from(value: Motherboard) -> Self {
        Self { inner: value }
    }
}

#[js_methods]
impl JsMotherboard {
    /// Name
    #[get]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// Vendor name
    #[get]
    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.inner.vendor_name()
    }

    /// Version
    #[get]
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.inner.version()
    }

    /// Serial number
    #[get]
    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.inner.serial_number()
    }

    /// Asset tag
    #[get]
    #[must_use]
    pub fn asset_tag(&self) -> Option<&str> {
        self.inner.asset_tag()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Motherboard", &self.inner)
    }
}

/// A hardware temperature sensor.
///
/// ```ts
/// const temperatureSensors = await system.hardware.listTemperatureSensors();
/// const temperatureSensor = temperatureSensors[0];
/// if (temperatureSensor) {
///   println(temperatureSensor.label, temperatureSensor.temperature);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsTemperatureSensor {
    inner: TemperatureSensor,
}

impl<'js> HostClass<'js> for JsTemperatureSensor {}

impl<'js> Trace<'js> for JsTemperatureSensor {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<TemperatureSensor> for JsTemperatureSensor {
    fn from(value: TemperatureSensor) -> Self {
        Self { inner: value }
    }
}

#[js_methods]
impl JsTemperatureSensor {
    /// Label
    #[get]
    #[must_use]
    pub fn label(&self) -> &str {
        self.inner.label()
    }

    /// ID
    #[get]
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.inner.id().as_deref()
    }

    /// Temperature
    #[get]
    #[must_use]
    pub fn temperature(&self) -> Option<f64> {
        self.inner.temperature().as_deref().copied()
    }

    /// Maximum temperature
    #[get]
    #[must_use]
    pub fn max_temperature(&self) -> Option<f64> {
        self.inner.max_temperature().as_deref().copied()
    }

    /// Critical temperature
    #[get]
    #[must_use]
    pub fn critical_temperature(&self) -> Option<f64> {
        self.inner.critical_temperature().as_deref().copied()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("TemperatureSensor", &self.inner)
    }
}
