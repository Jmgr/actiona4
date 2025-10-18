use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use eyre::Result;
use itertools::Itertools;
use sysinfo::Product;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{DisplayFields, OptionalDegreesCelsius, OptionalSystemString, display_list};

#[derive(Debug)]
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
pub struct Component {
    label: String,
    id: OptionalSystemString,
    temperature: OptionalDegreesCelsius,
    max_temperature: OptionalDegreesCelsius,
    critical_temperature: OptionalDegreesCelsius,
}

impl From<&sysinfo::Component> for Component {
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

impl Display for Component {
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

#[derive_where::derive_where(Debug)]
pub struct Hardware {
    #[derive_where(skip)]
    components: Arc<Mutex<sysinfo::Components>>,

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

impl Display for Hardware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display_if_some("name", &self.name)
            .display_if_some("family", &self.family)
            .display_if_some("serial_number", &self.serial_number)
            .display_if_some("stock_keeping_unit", &self.stock_keeping_unit)
            .display_if_some("uuid", &self.uuid)
            .display_if_some("version", &self.version)
            .display_if_some("vendor_name", &self.vendor_name)
            .display("motherboard", &self.motherboard)
            .display("components", display_list(&self.components()))
            .finish(f)
    }
}

impl Hardware {
    #[instrument(name = "hardware", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let components = task_tracker
            .spawn_blocking(sysinfo::Components::new_with_refreshed_list)
            .await?;

        Ok(Self {
            components: Arc::new(Mutex::new(components)),
            name: Product::name().into(),
            family: Product::family().into(),
            serial_number: Product::serial_number().into(),
            stock_keeping_unit: Product::stock_keeping_unit().into(),
            uuid: Product::uuid().into(),
            version: Product::version().into(),
            vendor_name: Product::vendor_name().into(),
            motherboard: Motherboard::default(),
            task_tracker,
        })
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    #[must_use]
    pub fn stock_keeping_unit(&self) -> Option<&str> {
        self.stock_keeping_unit.as_deref()
    }

    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    #[must_use]
    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    #[must_use]
    pub fn vendor_name(&self) -> Option<&str> {
        self.vendor_name.as_deref()
    }

    #[must_use]
    pub const fn motherboard(&self) -> &Motherboard {
        &self.motherboard
    }

    pub async fn refresh_components(&self, rescan: bool) -> Result<Vec<Component>> {
        let components = self.components.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut components = components.lock().unwrap();
                components.refresh(rescan);
                components.list().iter().map(Component::from).collect_vec()
            })
            .await?;
        Ok(result)
    }

    pub async fn refresh_component(&self, component: &Component) -> Result<Option<Component>> {
        let component_id = component.id.clone();
        let component_label = component.label.clone();
        let components = self.components.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut components = components.lock().unwrap();
                let component = if let Some(id) = component_id.as_ref() {
                    components
                        .list_mut()
                        .iter_mut()
                        .find(|c| c.id() == Some(id))
                } else {
                    components
                        .list_mut()
                        .iter_mut()
                        .find(|c| c.label() == component_label)
                }?;
                component.refresh();
                Some(Component::from(&*component))
            })
            .await?;
        Ok(result)
    }

    pub fn components(&self) -> Vec<Component> {
        let components = self.components.lock().unwrap();

        components.list().iter().map(Component::from).collect_vec()
    }
}
