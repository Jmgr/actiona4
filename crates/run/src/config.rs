use color_eyre::{Result, eyre::bail};
use config::Config;

pub async fn run(config: &Config, key: &str, value: Option<bool>) -> Result<()> {
    match (key, value) {
        ("update_check", Some(v)) => {
            config.settings_mut(|s| s.update_check = v).await?;
        }
        ("update_check", None) => {
            let v = config.settings(|s| s.update_check);
            println!("{v}");
        }
        ("telemetry", Some(v)) => {
            config.settings_mut(|s| s.set_telemetry(v)).await?;
        }
        ("telemetry", None) => {
            let v = config.settings(|s| s.telemetry.is_some());
            println!("{v}");
        }
        _ => bail!("unknown config key '{key}'. Valid keys: update_check, telemetry"),
    }

    Ok(())
}
