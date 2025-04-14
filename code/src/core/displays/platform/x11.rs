use std::sync::Arc;

use x11rb_async::protocol::randr::NotifyMask;

use super::Result;
use crate::runtime::Runtime;

#[derive(Debug)]
pub struct DisplaysImpl {}

impl DisplaysImpl {
    pub fn new(_runtime: Arc<Runtime>) -> Result<Self> {
        Ok(DisplaysImpl {})
    }

    pub fn randr_event_mask() -> NotifyMask {
        NotifyMask::SCREEN_CHANGE
    }
}
