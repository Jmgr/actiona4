use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use itertools::Itertools;
use sysinfo::Product;

use crate::core::system::normalize_string;

#[derive(Debug)]
pub struct Component {
    label: String,
    id: Option<String>,
    temperature: Option<f32>,
    max_temperature: Option<f32>,
    critical_temperature: Option<f32>,
}

impl Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(label: {}, id: {}, temperature: {}, max_temperature: {}, critical_temperature: {})",
            self.label,
            self.id.as_deref().unwrap_or_default(),
            self.temperature.unwrap_or_default(),
            self.max_temperature.unwrap_or_default(),
            self.critical_temperature.unwrap_or_default()
        )
    }
}

impl From<&sysinfo::Component> for Component {
    fn from(value: &sysinfo::Component) -> Self {
        Self {
            label: value.label().to_string(),
            id: normalize_string(value.id().map(|id| id.to_string())),
            temperature: value.temperature(),
            max_temperature: value.max(),
            critical_temperature: value.critical(),
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Hardware {
    #[derive_where(skip)]
    components: Arc<Mutex<sysinfo::Components>>,

    name: Option<String>,
    family: Option<String>,
    serial_number: Option<String>,
    stock_keeping_unit: Option<String>,
    uuid: Option<String>,
    version: Option<String>,
    vendor_name: Option<String>,
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            components: Arc::new(Mutex::new(sysinfo::Components::new())),
            name: normalize_string(Product::name()),
            family: normalize_string(Product::family()),
            serial_number: normalize_string(Product::serial_number()),
            stock_keeping_unit: normalize_string(Product::stock_keeping_unit()),
            uuid: normalize_string(Product::uuid()),
            version: normalize_string(Product::version()),
            vendor_name: normalize_string(Product::vendor_name()),
        }
    }
}

impl Hardware {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    pub fn stock_keeping_unit(&self) -> Option<&str> {
        self.stock_keeping_unit.as_deref()
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn vendor_name(&self) -> Option<&str> {
        self.vendor_name.as_deref()
    }

    pub fn components(&self) -> Vec<Component> {
        let mut components = self.components.lock().unwrap();
        components.refresh(true);

        components
            .list()
            .iter()
            .map(|component| component.into())
            .collect_vec()
    }
}
