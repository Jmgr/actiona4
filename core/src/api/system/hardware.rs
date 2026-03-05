use std::{fmt::Display, sync::Arc};

use color_eyre::Result;
use derive_where::derive_where;
use itertools::Itertools;
use parking_lot::Mutex;
use sysinfo::Product;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{
    OptionalDegreesCelsius, OptionalSystemString,
    display::{DisplayFields, display_list},
};

#[derive(Clone, Debug)]
pub struct Motherboard {
    name: OptionalSystemString,
    vendor: OptionalSystemString,
    version: OptionalSystemString,
    serial_number: OptionalSystemString,
    asset_tag: OptionalSystemString,
}

impl Default for Motherboard {
    fn default() -> Self {
        sysinfo::Motherboard::new().map_or_else(
            || Self {
                name: OptionalSystemString::none(),
                vendor: OptionalSystemString::none(),
                version: OptionalSystemString::none(),
                serial_number: OptionalSystemString::none(),
                asset_tag: OptionalSystemString::none(),
            },
            |motherboard| Self {
                name: motherboard.name().into(),
                vendor: motherboard.vendor_name().into(),
                version: motherboard.version().into(),
                serial_number: motherboard.serial_number().into(),
                asset_tag: motherboard.asset_tag().into(),
            },
        )
    }
}

impl Display for Motherboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("name", &self.name)
            .display_if_some("vendor", &self.vendor)
            .display_if_some("version", &self.version)
            .display_if_some("serial_number", &self.serial_number)
            .display_if_some("asset_tag", &self.asset_tag)
            .finish(f)
    }
}

impl Motherboard {
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.vendor.as_deref()
    }

    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    #[must_use]
    pub fn asset_tag(&self) -> Option<&str> {
        self.asset_tag.as_deref()
    }
}

#[derive(Debug)]
pub struct TemperatureSensor {
    label: String,
    id: OptionalSystemString,
    temperature: OptionalDegreesCelsius,
    max_temperature: OptionalDegreesCelsius,
    critical_temperature: OptionalDegreesCelsius,
}

impl From<&sysinfo::Component> for TemperatureSensor {
    fn from(value: &sysinfo::Component) -> Self {
        Self {
            label: value.label().to_string(),
            id: value.id().into(),
            temperature: value.temperature().into(),
            max_temperature: value.max().into(),
            critical_temperature: value.critical().into(),
        }
    }
}

impl Display for TemperatureSensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            DisplayFields::default()
                .display("label", &self.label)
                .display_if_some("id", &self.id)
                .display_if_some("temperature", &self.temperature)
                .display_if_some("max_temperature", &self.max_temperature)
                .display_if_some("critical_temperature", &self.critical_temperature)
                .finish(f)
        } else {
            DisplayFields::default()
                .display("label", &self.label)
                .display_if_some("temperature", &self.temperature)
                .display_if_some("max_temperature", &self.max_temperature)
                .display_if_some("critical_temperature", &self.critical_temperature)
                .finish(f)
        }
    }
}

impl TemperatureSensor {
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub const fn id(&self) -> &OptionalSystemString {
        &self.id
    }

    #[must_use]
    pub const fn temperature(&self) -> &OptionalDegreesCelsius {
        &self.temperature
    }

    #[must_use]
    pub const fn max_temperature(&self) -> &OptionalDegreesCelsius {
        &self.max_temperature
    }

    #[must_use]
    pub const fn critical_temperature(&self) -> &OptionalDegreesCelsius {
        &self.critical_temperature
    }
}

#[derive_where(Debug)]
struct HardwareInner {
    #[derive_where(skip)]
    temperature_sensors: Arc<Mutex<sysinfo::Components>>,

    name: OptionalSystemString,
    family: OptionalSystemString,
    serial_number: OptionalSystemString,
    stock_keeping_unit: OptionalSystemString,
    uuid: OptionalSystemString,
    version: OptionalSystemString,
    vendor_name: OptionalSystemString,
    motherboard: Motherboard,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

#[derive(Clone, Debug)]
pub struct Hardware {
    inner: Arc<HardwareInner>,
}

impl Display for Hardware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display_if_some("name", &self.inner.name)
            .display_if_some("family", &self.inner.family)
            .display_if_some("serial_number", &self.inner.serial_number)
            .display_if_some("stock_keeping_unit", &self.inner.stock_keeping_unit)
            .display_if_some("uuid", &self.inner.uuid)
            .display_if_some("version", &self.inner.version)
            .display_if_some("vendor_name", &self.inner.vendor_name)
            .display("motherboard", &self.inner.motherboard)
            .display(
                "temperature_sensors",
                display_list(&self.temperature_sensors()),
            )
            .finish(f)
    }
}

impl Hardware {
    #[instrument(name = "hardware", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let temperature_sensors = task_tracker
            .spawn_blocking(sysinfo::Components::new)
            .await?;

        Ok(Self {
            inner: Arc::new(HardwareInner {
                temperature_sensors: Arc::new(Mutex::new(temperature_sensors)),
                name: Product::name().into(),
                family: Product::family().into(),
                serial_number: Product::serial_number().into(),
                stock_keeping_unit: Product::stock_keeping_unit().into(),
                uuid: Product::uuid().into(),
                version: Product::version().into(),
                vendor_name: Product::vendor_name().into(),
                motherboard: Motherboard::default(),
                task_tracker,
            }),
        })
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.inner.family.as_deref()
    }

    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.inner.serial_number.as_deref()
    }

    #[must_use]
    pub fn stock_keeping_unit(&self) -> Option<&str> {
        self.inner.stock_keeping_unit.as_deref()
    }

    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.inner.version.as_deref()
    }

    #[must_use]
    pub fn uuid(&self) -> Option<&str> {
        self.inner.uuid.as_deref()
    }

    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.inner.vendor_name.as_deref()
    }

    #[must_use]
    pub fn motherboard(&self) -> &Motherboard {
        &self.inner.motherboard
    }

    pub async fn refresh_temperature_sensors(
        &self,
        rescan: bool,
    ) -> Result<Vec<TemperatureSensor>> {
        let temperature_sensors = self.inner.temperature_sensors.clone();
        let result = self
            .inner
            .task_tracker
            .spawn_blocking(move || {
                let mut temperature_sensors = temperature_sensors.lock();
                temperature_sensors.refresh(rescan);
                temperature_sensors
                    .list()
                    .iter()
                    .map(TemperatureSensor::from)
                    .collect_vec()
            })
            .await?;
        Ok(result)
    }

    pub async fn refresh_temperature_sensor(
        &self,
        temperature_sensor: &TemperatureSensor,
    ) -> Result<Option<TemperatureSensor>> {
        let temperature_sensor_id = temperature_sensor.id.clone();
        let temperature_sensor_label = temperature_sensor.label.clone();
        let temperature_sensors = self.inner.temperature_sensors.clone();
        let result = self
            .inner
            .task_tracker
            .spawn_blocking(move || {
                let mut temperature_sensors = temperature_sensors.lock();
                let temperature_sensor = if let Some(id) = temperature_sensor_id.as_ref() {
                    temperature_sensors
                        .list_mut()
                        .iter_mut()
                        .find(|c| c.id() == Some(id))
                } else {
                    temperature_sensors
                        .list_mut()
                        .iter_mut()
                        .find(|c| c.label() == temperature_sensor_label)
                }?;
                temperature_sensor.refresh();
                Some(TemperatureSensor::from(&*temperature_sensor))
            })
            .await?;
        Ok(result)
    }

    pub fn temperature_sensors(&self) -> Vec<TemperatureSensor> {
        let temperature_sensors = self.inner.temperature_sensors.lock();

        temperature_sensors
            .list()
            .iter()
            .map(TemperatureSensor::from)
            .collect_vec()
    }
}
