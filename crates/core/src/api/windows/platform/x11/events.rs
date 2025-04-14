use std::{collections::HashMap, sync::Arc};

use color_eyre::Result;
use parking_lot::Mutex;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use x11rb::protocol::xproto::{ChangeWindowAttributesAux, EventMask};
use x11rb_async::protocol::xproto::ConnectionExt as _;

use crate::{
    api::windows::platform::x11::WindowHandle,
    platform::x11::X11Connection,
    runtime::events::{AllSignals, Topic, TopicWrapper},
};

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Closed(WindowHandle),
    Hidden(WindowHandle),
    Visible(WindowHandle),
}

pub struct WindowEventsTopic {
    window_id: u32,
    x11_connection: Arc<X11Connection>,
}

impl Topic for WindowEventsTopic {
    type T = WindowEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        set_mask(
            &self.x11_connection,
            self.window_id,
            EventMask::STRUCTURE_NOTIFY,
        )
        .await
    }

    async fn on_stop(&self) -> Result<()> {
        set_mask(&self.x11_connection, self.window_id, EventMask::NO_EVENT).await
    }
}

async fn set_mask(x11_connection: &X11Connection, window_id: u32, mask: EventMask) -> Result<()> {
    let connection = x11_connection.async_connection();
    connection
        .change_window_attributes(
            window_id,
            &ChangeWindowAttributesAux::new().event_mask(mask),
        )
        .await?;
    x11rb_async::connection::Connection::flush(connection).await?;
    Ok(())
}

/// Manages per-window `STRUCTURE_NOTIFY` subscriptions using the `Topic`/`TopicWrapper` pattern.
/// Each window gets its own `TopicWrapper<WindowEventsTopic>`, created on first subscription
/// and removed when no subscribers remain.
#[derive(Clone, Debug)]
pub struct WindowSubscriptions {
    x11_connection: Arc<X11Connection>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
    topics: Arc<Mutex<HashMap<u32, TopicWrapper<WindowEventsTopic>>>>,
}

impl WindowSubscriptions {
    pub fn new(
        x11_connection: Arc<X11Connection>,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Self {
        Self {
            x11_connection,
            cancellation_token,
            task_tracker,
            topics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn get_or_create(&self, window_id: u32) -> TopicWrapper<WindowEventsTopic> {
        let mut topics = self.topics.lock();
        topics
            .entry(window_id)
            .or_insert_with(|| {
                TopicWrapper::new(
                    WindowEventsTopic {
                        window_id,
                        x11_connection: self.x11_connection.clone(),
                    },
                    self.cancellation_token.clone(),
                    self.task_tracker.clone(),
                )
            })
            .clone()
    }
}
