use color_eyre::{Result, eyre::eyre};
use tracing::instrument;

use crate::{
    api::{point::Point, rect::Rect},
    cancel_on,
    runtime::Runtime,
};

#[instrument(skip_all)]
pub async fn ask_rect(runtime: &Runtime) -> Result<Option<Rect>> {
    let selection = runtime.extensions().selection().await?;
    let Some(selection) = selection else {
        return Err(eyre!("selection extension is not available"));
    };

    cancel_on(&runtime.cancellation_token(), selection.select_rect()).await?
}

#[instrument(skip_all)]
pub async fn ask_position(runtime: &Runtime) -> Result<Option<Point>> {
    let selection = runtime.extensions().selection().await?;
    let Some(selection) = selection else {
        return Err(eyre!("selection extension is not available"));
    };

    cancel_on(&runtime.cancellation_token(), selection.select_position()).await?
}
