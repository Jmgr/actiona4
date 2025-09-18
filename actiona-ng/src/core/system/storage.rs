use version_number::Version;

use crate::core::system::normalize_string;

#[derive(Debug)]
pub struct Storage {
    name: Option<String>,
    vendor_name: Option<String>,
    version: Option<Version>,
    serial_number: Option<String>,
    asset_tag: Option<String>,
}

impl Default for Storage {
    fn default() -> Self {
        if let Some(motherboard) = sysinfo::Motherboard::new() {
            Self {
                name: normalize_string(motherboard.name()),
                vendor_name: normalize_string(motherboard.vendor_name()),
                version: motherboard
                    .version()
                    .and_then(|version| Version::parse(&version).ok()),
                serial_number: normalize_string(motherboard.serial_number()),
                asset_tag: normalize_string(motherboard.asset_tag()),
            }
        } else {
            Self {
                name: None,
                vendor_name: None,
                version: None,
                serial_number: None,
                asset_tag: None,
            }
        }
    }
}

impl Storage {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn vendor_name(&self) -> Option<&str> {
        self.vendor_name.as_deref()
    }

    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    pub fn asset_tag(&self) -> Option<&str> {
        self.asset_tag.as_deref()
    }
}
