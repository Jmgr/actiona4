#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

use color_eyre::Result;
use tokio_util::sync::CancellationToken;

pub use super::Button;
use crate::{api::mouse::ButtonConditions, runtime::events::MouseButtonEvent};

pub trait MouseImplTrait {
    async fn is_button_pressed(&self, button: Button) -> Result<bool>;
    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent>;
}
