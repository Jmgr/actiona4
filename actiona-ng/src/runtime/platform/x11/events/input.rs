use std::sync::{Arc, Mutex};

use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use x11rb::protocol::xinput::{Device, EventMask, XIEventMask};
use x11rb_async::{connection::Connection, protocol::xinput::xi_select_events};

use crate::{
    core::point::Point,
    platform::x11::X11Connection,
    runtime::events::{
        AllSignals, Control, LatestOnlySignals, MouseButtonEvent, ReceiverGuard, Topic,
    },
};

#[derive(Clone, Default)]
pub struct InputMask {
    inner: Arc<Mutex<XIEventMask>>,
}

impl InputMask {
    pub fn set(&self, mask: XIEventMask) {
        let mut inner = self.inner.lock().unwrap();
        *inner |= mask;
    }

    pub fn remove(&self, mask: XIEventMask) {
        let mut inner = self.inner.lock().unwrap();
        *inner = inner.remove(mask);
    }

    pub fn to_vec(&self) -> Vec<XIEventMask> {
        let inner = self.inner.lock().unwrap();
        vec![*inner]
    }
}

async fn apply_mask_on_control(
    name: &'static str,
    mask: XIEventMask,
    x11_connection: Arc<X11Connection>,
    input_mask: Arc<InputMask>,
    mut control_receiver: watch::Receiver<Control>,
    cancellation_token: CancellationToken,
) {
    let connection = x11_connection.async_connection();
    loop {
        select! {
            _ = cancellation_token.cancelled() => break,
            changed = control_receiver.changed() => {
                if changed.is_err() { break; } // sender dropped
            }
        }

        let control = *control_receiver.borrow_and_update();

        info!("{name}: {}", control);

        match control {
            Control::Enable => input_mask.set(mask),
            Control::Disable => input_mask.remove(mask),
        }

        if let Err(err) = xi_select_events(
            connection,
            x11_connection.screen().root,
            &[EventMask {
                deviceid: Device::ALL_MASTER.into(),
                mask: input_mask.to_vec(),
            }],
        )
        .await
        {
            error!("{name}: xi_select_events failed: {err}");
        }

        if let Err(err) = connection.flush().await {
            error!("flush failed: {err}");
        }
    }
}

#[derive(Debug)]
pub struct MouseButtonsTopic(Topic<MouseButtonEvent, AllSignals<MouseButtonEvent>>);

impl MouseButtonsTopic {
    pub fn new(
        x11_connection: Arc<X11Connection>,
        input_mask: Arc<InputMask>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let (topic, control_receiver) = Topic::new(AllSignals::new());
        task_tracker.spawn(apply_mask_on_control(
            "mouse_buttons",
            XIEventMask::RAW_BUTTON_PRESS | XIEventMask::RAW_BUTTON_RELEASE,
            x11_connection,
            input_mask,
            control_receiver,
            cancellation_token,
        ));
        Self(topic)
    }

    pub fn publish(&self, value: MouseButtonEvent) {
        self.0.publish(value);
    }

    pub fn subscribe(&self) -> ReceiverGuard<MouseButtonEvent, AllSignals<MouseButtonEvent>> {
        self.0.subscribe()
    }
}

#[derive(Debug)]
pub struct MouseMoveTopic(Topic<Point, LatestOnlySignals<Point>>);

impl MouseMoveTopic {
    pub fn new(
        x11_connection: Arc<X11Connection>,
        input_mask: Arc<InputMask>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let (topic, control_receiver) = Topic::new(LatestOnlySignals::new());
        task_tracker.spawn(apply_mask_on_control(
            "mouse_move",
            XIEventMask::MOTION,
            x11_connection,
            input_mask,
            control_receiver,
            cancellation_token,
        ));
        Self(topic)
    }

    pub fn publish(&self, value: Point) {
        self.0.publish(value);
    }

    pub fn subscribe(&self) -> ReceiverGuard<Point, LatestOnlySignals<Point>> {
        self.0.subscribe()
    }
}
