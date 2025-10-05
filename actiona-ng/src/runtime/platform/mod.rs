use tokio::sync::broadcast::Receiver;

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;
