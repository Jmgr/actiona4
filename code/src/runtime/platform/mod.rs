use tokio::sync::broadcast::Receiver;

use super::RecordEvent;

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub trait RuntimePlatform {
    fn subcribe_events(&self) -> Receiver<RecordEvent>;
}
