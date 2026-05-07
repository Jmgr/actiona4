use color_eyre::{
    Result,
    eyre::{OptionExt, WrapErr},
};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    api::{point::Point, rect::Rect},
    cancel_on,
    error::CommonError,
    runtime::Runtime,
};

#[instrument(skip_all)]
pub async fn ask_rect(runtime: &Runtime, cancellation: CancellationToken) -> Result<Option<Rect>> {
    let selection = runtime
        .extensions()
        .selection()
        .ok_or_eyre("selection extension is not available")?;

    match cancel_on(&cancellation, selection.select_rect()).await {
        Ok(selection) => selection.wrap_err("selection overlay failed"),
        Err(error)
            if error
                .downcast_ref::<CommonError>()
                .is_some_and(CommonError::is_cancelled) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.wrap_err("waiting for selection overlay failed")),
    }
}

#[instrument(skip_all)]
pub async fn ask_position(
    runtime: &Runtime,
    cancellation: CancellationToken,
) -> Result<Option<Point>> {
    let selection = runtime
        .extensions()
        .selection()
        .ok_or_eyre("selection extension is not available")?;

    match cancel_on(&cancellation, selection.select_position()).await {
        Ok(selection) => selection.wrap_err("selection overlay failed"),
        Err(error)
            if error
                .downcast_ref::<CommonError>()
                .is_some_and(CommonError::is_cancelled) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.wrap_err("waiting for selection overlay failed")),
    }
}
