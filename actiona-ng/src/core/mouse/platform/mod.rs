#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

use tokio_util::sync::CancellationToken;

pub use super::{Button, Result};
use crate::{core::mouse::ButtonConditions, runtime::events::MouseButtonEvent};

pub trait MouseImplTrait {
    async fn is_button_pressed(&self, button: Button) -> Result<bool>;
    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent>;
}
