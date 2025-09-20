use crate::types::OptionalString;

#[derive(Debug)]
pub struct Motherboard {
    name: OptionalString,
    vendor_name: OptionalString,
    version: OptionalString,
    serial_number: OptionalString,
    asset_tag: OptionalString,
}

impl Default for Motherboard {
    fn default() -> Self {
        if let Some(motherboard) = sysinfo::Motherboard::new() {
            Self {
                name: motherboard.name().into(),
                vendor_name: motherboard.vendor_name().into(),
                version: motherboard.version().into(),
                serial_number: motherboard.serial_number().into(),
                asset_tag: motherboard.asset_tag().into(),
            }
        } else {
            Self {
                name: OptionalString::none(),
                vendor_name: OptionalString::none(),
                version: OptionalString::none(),
                serial_number: OptionalString::none(),
                asset_tag: OptionalString::none(),
            }
        }
    }
}

impl Motherboard {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn vendor_name(&self) -> Option<&str> {
        self.vendor_name.as_deref()
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    pub fn asset_tag(&self) -> Option<&str> {
        self.asset_tag.as_deref()
    }
}
