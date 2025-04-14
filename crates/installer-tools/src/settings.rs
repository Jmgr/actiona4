use std::future::Future;

use config::Config;
use eyre::Result;
use tokio::runtime::Builder;

pub fn get_update_check_enabled() -> Result<bool> {
    with_runtime(async move {
        let config = Config::new().await?;
        Ok(config.settings(|settings| settings.update_check))
    })
}

pub fn set_update_check_enabled(update_check_enabled: bool) -> Result<()> {
    with_runtime(async move {
        let config = Config::new().await?;
        config
            .settings_mut(|settings| settings.update_check = update_check_enabled)
            .await?;
        Ok(())
    })
}

pub fn get_telemetry_enabled() -> Result<bool> {
    with_runtime(async move {
        let config = Config::new().await?;
        Ok(config.settings(|settings| settings.telemetry.is_some()))
    })
}

pub fn set_telemetry_enabled(telemetry_enabled: bool) -> Result<()> {
    with_runtime(async move {
        let config = Config::new().await?;
        config
            .settings_mut(|settings| settings.set_telemetry(telemetry_enabled))
            .await?;
        Ok(())
    })
}

fn with_runtime<F, R>(future: F) -> Result<R>
where
    F: Future<Output = Result<R>>,
{
    let runtime = Builder::new_current_thread().enable_all().build()?;

    runtime.block_on(future)
}
