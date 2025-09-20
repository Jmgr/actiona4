use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use itertools::Itertools;
use sysinfo::Product;

use crate::types::{OptionalDegreesCelsius, OptionalString};

#[derive_where::derive_where(Debug)]
pub struct Component {
    #[derive_where(skip)]
    components: Arc<Mutex<sysinfo::Components>>,

    label: String,
    id: OptionalString,
    temperature: OptionalDegreesCelsius,
    max_temperature: OptionalDegreesCelsius,
    critical_temperature: OptionalDegreesCelsius,
}

impl Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(label: {}, id: {}, temperature: {}, max_temperature: {}, critical_temperature: {})",
            self.label, self.id, self.temperature, self.max_temperature, self.critical_temperature
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
            id: component.id().into(),
            temperature: component.temperature().into(),
            max_temperature: component.max().into(),
            critical_temperature: component.critical().into(),
        }
    }

    pub fn refresh(&mut self) -> bool {
        let mut components = self.components.lock().unwrap();

        let component = if let Some(id) = self.id.as_ref() {
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
            self.id = component.id().into();
        }
        self.temperature = component.temperature().into();
        self.max_temperature = component.max().into();
        self.critical_temperature = component.critical().into();

        true
    }
}

#[derive_where::derive_where(Debug)]
pub struct Hardware {
    #[derive_where(skip)]
    components: Arc<Mutex<sysinfo::Components>>,

    name: OptionalString,
    family: OptionalString,
    serial_number: OptionalString,
    stock_keeping_unit: OptionalString,
    uuid: OptionalString,
    version: OptionalString,
    vendor_name: OptionalString,
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            components: Arc::new(Mutex::new(sysinfo::Components::new())),
            name: Product::name().into(),
            family: Product::family().into(),
            serial_number: Product::serial_number().into(),
            stock_keeping_unit: Product::stock_keeping_unit().into(),
            uuid: Product::uuid().into(),
            version: Product::version().into(),
            vendor_name: Product::vendor_name().into(),
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
