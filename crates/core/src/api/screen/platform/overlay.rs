use color_eyre::Result;
use tracing::instrument;

use crate::{
    api::{point::Point, rect::Rect},
    cancel_on,
    runtime::Runtime,
};

#[instrument(skip_all)]
pub async fn ask_rect(runtime: &Runtime) -> Result<Option<Rect>> {
    let selection = runtime.extensions().selection().await?;

    cancel_on(&runtime.cancellation_token(), selection.select_rect()).await?
}

#[instrument(skip_all)]
pub async fn ask_position(runtime: &Runtime) -> Result<Option<Point>> {
    let selection = runtime.extensions().selection().await?;

    cancel_on(&runtime.cancellation_token(), selection.select_position()).await?
}
