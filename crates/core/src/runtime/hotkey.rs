use std::{
    fmt::{self, Display},
    str::FromStr,
};

use color_eyre::{
    Report, Result,
    eyre::{bail, eyre},
};
use strum::IntoEnumIterator;

use crate::api::keyboard::js::JsStandardKey;

/// Default global hotkey that stops script execution.
pub const DEFAULT_STOP_HOTKEY: &str = "Ctrl+Alt+Shift+Q";

/// A global keyboard shortcut: a set of modifiers and a single key, parsed from strings such as
/// `"Ctrl+Alt+Shift+Q"` or `"Ctrl+F12"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    pub key: enigo::Key,
    key_name: String,
}

impl Hotkey {
    /// Parses a hotkey setting, where `none` (or an empty string) disables the hotkey.
    pub fn parse_optional(value: &str) -> Result<Option<Self>> {
        let value = value.trim();
        if value.is_empty() || value.eq_ignore_ascii_case("none") {
            return Ok(None);
        }

        value.parse().map(Some)
    }

    fn parse_key(name: &str) -> Result<(enigo::Key, String)> {
        let mut chars = name.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            // Letters are stored lowercase: that is the unshifted keysym X11 needs to find the
            // keycode, and Windows maps both cases to the same virtual key.
            let c = c.to_lowercase().next().unwrap_or(c);
            return Ok((enigo::Key::Unicode(c), c.to_uppercase().collect()));
        }

        let standard_key = JsStandardKey::iter()
            .find(|key| key.to_string().eq_ignore_ascii_case(name))
            .ok_or_else(|| eyre!("unknown key name: {name:?}"))?;
        let key = enigo::Key::try_from(standard_key)
            .map_err(|_| eyre!("key {name:?} is not supported on this platform"))?;

        Ok((key, standard_key.to_string()))
    }
}

enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

impl Modifier {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Self::Ctrl,
            "alt" => Self::Alt,
            "shift" => Self::Shift,
            "meta" | "super" | "win" | "windows" => Self::Meta,
            _ => return None,
        })
    }
}

impl FromStr for Hotkey {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self> {
        let mut parts = value.split('+').map(str::trim);
        let key_part = parts.next_back().unwrap_or_default();
        if key_part.is_empty() || Modifier::from_name(key_part).is_some() {
            bail!(
                "invalid hotkey {value:?}: expected modifiers followed by a key, such as {DEFAULT_STOP_HOTKEY}"
            );
        }

        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut meta = false;
        for modifier in parts {
            let flag = match Modifier::from_name(modifier) {
                Some(Modifier::Ctrl) => &mut ctrl,
                Some(Modifier::Alt) => &mut alt,
                Some(Modifier::Shift) => &mut shift,
                Some(Modifier::Meta) => &mut meta,
                None => bail!(
                    "invalid hotkey {value:?}: unknown modifier {modifier:?} (expected Ctrl, Alt, Shift or Meta)"
                ),
            };
            if *flag {
                bail!("invalid hotkey {value:?}: modifier {modifier:?} is repeated");
            }
            *flag = true;
        }

        let (key, key_name) =
            Self::parse_key(key_part).map_err(|err| eyre!("invalid hotkey {value:?}: {err}"))?;

        Ok(Self {
            ctrl,
            alt,
            shift,
            meta,
            key,
            key_name,
        })
    }
}

/// Canonical form, identical for every spelling of the same combination. Instances use it to
/// find each other, so that stopping one stops every instance that shares the hotkey.
impl Display for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (enabled, name) in [
            (self.ctrl, "Ctrl"),
            (self.alt, "Alt"),
            (self.shift, "Shift"),
            (self.meta, "Meta"),
        ] {
            if enabled {
                write!(f, "{name}+")?;
            }
        }

        write!(f, "{}", self.key_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default() {
        let hotkey: Hotkey = DEFAULT_STOP_HOTKEY.parse().unwrap();
        assert!(hotkey.ctrl && hotkey.alt && hotkey.shift && !hotkey.meta);
        assert_eq!(hotkey.key, enigo::Key::Unicode('q'));
        assert_eq!(hotkey.to_string(), DEFAULT_STOP_HOTKEY);
    }

    #[test]
    fn display_is_canonical() {
        let hotkey: Hotkey = " shift + CONTROL+super+alt+q ".parse().unwrap();
        assert_eq!(hotkey.to_string(), "Ctrl+Alt+Shift+Meta+Q");
    }

    #[test]
    fn parses_named_keys_case_insensitively() {
        let hotkey: Hotkey = "ctrl+f12".parse().unwrap();
        assert_eq!(hotkey.key, enigo::Key::F12);
        assert_eq!(hotkey.to_string(), "Ctrl+F12");

        let hotkey: Hotkey = "Pause".parse().unwrap();
        assert!(!hotkey.ctrl && !hotkey.alt && !hotkey.shift && !hotkey.meta);
        assert_eq!(hotkey.key, enigo::Key::Pause);
    }

    #[test]
    fn rejects_invalid_hotkeys() {
        for value in [
            "",
            "Ctrl+",
            "Ctrl+Alt",
            "Hyper+Q",
            "Ctrl+Ctrl+Q",
            "Ctrl+NotAKey",
        ] {
            assert!(
                value.parse::<Hotkey>().is_err(),
                "{value:?} should not parse"
            );
        }
    }

    #[test]
    fn none_disables() {
        assert_eq!(Hotkey::parse_optional("none").unwrap(), None);
        assert_eq!(Hotkey::parse_optional(" ").unwrap(), None);
        assert!(Hotkey::parse_optional("Ctrl+Q").unwrap().is_some());
    }
}
