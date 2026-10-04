use actiona_core::runtime::hotkey::{DEFAULT_STOP_HOTKEY, Hotkey};
use color_eyre::{Result, eyre::Context};
use config::CommonConfig;

use crate::args::ConfigKey;

fn parse_bool(value: &str) -> Result<bool> {
    value
        .parse()
        .context("invalid value: expected true or false")
}

pub async fn run(config: &CommonConfig, key: ConfigKey, value: Option<&str>) -> Result<()> {
    match (key, value) {
        (ConfigKey::UpdateCheck, Some(v)) => {
            let v = parse_bool(v)?;
            config.settings_mut(|s| s.update_check = v).await?;
        }
        (ConfigKey::UpdateCheck, None) => {
            let v = config.settings(|s| s.update_check);
            println!("{v}");
        }
        (ConfigKey::Telemetry, Some(v)) => {
            let v = parse_bool(v)?;
            config.settings_mut(|s| s.set_telemetry(v)).await?;
        }
        (ConfigKey::Telemetry, None) => {
            let v = config.settings(|s| s.telemetry.is_some());
            println!("{v}");
        }
        (ConfigKey::StopHotkey, Some(v)) => {
            let v = if v.eq_ignore_ascii_case("default") {
                None
            } else {
                Some(
                    Hotkey::parse_optional(v)?
                        .map_or_else(|| "none".to_owned(), |hotkey| hotkey.to_string()),
                )
            };
            config.settings_mut(|s| s.stop_hotkey = v).await?;
        }
        (ConfigKey::StopHotkey, None) => {
            let v = config.settings(|s| s.stop_hotkey.clone());
            println!("{}", v.as_deref().unwrap_or(DEFAULT_STOP_HOTKEY));
        }
    }

    Ok(())
}
