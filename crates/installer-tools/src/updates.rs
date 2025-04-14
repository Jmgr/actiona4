use std::future::Future;

use config::Config;
use eyre::{Result, eyre};
use updater::Updater;
use versions::SemVer;

const APPLICATION_NAME: &str = "actiona-run";
const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct UpdateCheckResult {
    pub version: String,
    pub download_url: String,
}

pub fn check_for_update() -> Result<Option<UpdateCheckResult>> {
    with_runtime(async move {
        let config = Config::new().await?;
        let app_version = SemVer::new(APPLICATION_VERSION)
            .ok_or_else(|| eyre!("invalid app version: {APPLICATION_VERSION}"))?;
        let version_info =
            Updater::check_once(&config, APPLICATION_NAME, app_version.clone(), "installer")
                .await?;
        let version_info = version_info.filter(|version_info| version_info.version > app_version);

        Ok(version_info.map(|version_info| UpdateCheckResult {
            version: version_info.version.to_string(),
            download_url: version_info.download_url,
        }))
    })
}

fn with_runtime<F>(future: F) -> Result<Option<UpdateCheckResult>>
where
    F: Future<Output = Result<Option<UpdateCheckResult>>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(future)
}
