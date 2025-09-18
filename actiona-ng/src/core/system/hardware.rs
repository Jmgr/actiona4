use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use itertools::Itertools;
use sysinfo::Product;

use crate::core::system::normalize_string;

#[derive_where::derive_where(Debug)]
pub struct Component {
    #[derive_where(skip)]
    components: Arc<Mutex<sysinfo::Components>>,

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

impl Component {
    pub fn new(
        components: Arc<Mutex<sysinfo::Components>>,
        component: &sysinfo::Component,
    ) -> Self {
        Self {
            components,
            label: component.label().to_string(),
            id: normalize_string(component.id().map(|id| id.to_string())),
            temperature: component.temperature(),
            max_temperature: component.max(),
            critical_temperature: component.critical(),
        }
    }

    pub fn refresh(&mut self) -> bool {
        let mut components = self.components.lock().unwrap();

        let component = if let Some(id) = self.id.as_deref() {
            components
                .list_mut()
                .iter_mut()
                .find(|c| c.id() == Some(id))
        } else {
            components
                .list_mut()
                .iter_mut()
                .find(|c| c.label() == self.label)
        };

        let Some(component) = component else {
            return false;
        };

        component.refresh();

        self.label = component.label().to_string();
        if self.id.is_none() {
            // Only overwrite id if we don't already have one
            self.id = normalize_string(component.id().map(|id| id.to_string()));
        }
        self.temperature = component.temperature();
        self.max_temperature = component.max();
        self.critical_temperature = component.critical();

        true
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
            .map(|component| Component::new(self.components.clone(), component))
            .collect_vec()
    }
}
