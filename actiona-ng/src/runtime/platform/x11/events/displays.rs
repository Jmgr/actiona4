use std::sync::Arc;

use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use x11rb::protocol::randr::NotifyMask;
use x11rb_async::{connection::Connection, protocol::randr::ConnectionExt};

use crate::{
    platform::x11::X11Connection,
    runtime::events::{Control, DisplayInfoVec, LatestOnlySignals, ReceiverGuard, Topic},
};

#[derive(Debug)]
pub struct ScreenChangeTopic(Topic<DisplayInfoVec, LatestOnlySignals<DisplayInfoVec>>);

impl ScreenChangeTopic {
    pub fn new(
        x11_connection: Arc<X11Connection>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let (topic, mut control_receiver) = Topic::new(LatestOnlySignals::new());
        task_tracker.spawn(async move {
            let connection = x11_connection.async_connection();
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    changed = control_receiver.changed() => {
                        if changed.is_err() { break; } // sender dropped
                        let control = *control_receiver.borrow_and_update();

                        info!("ScreenChangeTopic: {}", control);

                        let mask = match control {
                            Control::Enable  => NotifyMask::SCREEN_CHANGE,
                            Control::Disable => NotifyMask::default(),
                        };

                        if let Err(err) = connection
                                .randr_select_input(x11_connection.screen().root, mask)
                                .await {
                            error!("randr_select_input failed: {err}");
                        }

                        if let Err(err) = connection.flush().await {
                            error!("flush failed: {err}");
                        }
                    }
                }
            }
        });
        Self(topic)
    }

    pub fn publish(&self, value: DisplayInfoVec) {
        self.0.publish(value);
    }

    pub fn receiver(&self) -> ReceiverGuard<DisplayInfoVec, LatestOnlySignals<DisplayInfoVec>> {
        self.0.subscribe()
    }
}
