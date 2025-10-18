use std::sync::{Arc, Mutex};

use derive_more::Constructor;
use tracing::error;
use x11rb::protocol::xinput::{Device, EventMask, XIEventMask};
use x11rb_async::{connection::Connection, protocol::xinput::xi_select_events};

use crate::{
    core::point::Point,
    platform::x11::X11Connection,
    runtime::events::{AllSignals, LatestOnlySignals, MouseButtonEvent, Topic},
};

#[derive(Clone, Debug, Default)]
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

    #[must_use]
    pub fn to_vec(&self) -> Vec<XIEventMask> {
        let inner = self.inner.lock().unwrap();
        vec![*inner]
    }
}

async fn apply_mask(
    name: &'static str,
    x11_connection: Arc<X11Connection>,
    input_mask: Arc<InputMask>,
) {
    let connection = x11_connection.async_connection();

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

#[derive(Debug, Constructor)]
pub struct MouseButtonsTopic {
    x11_connection: Arc<X11Connection>,
    input_mask: Arc<InputMask>,
}

impl Topic for MouseButtonsTopic {
    type T = MouseButtonEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) {
        self.input_mask
            .set(XIEventMask::RAW_BUTTON_PRESS | XIEventMask::RAW_BUTTON_RELEASE);

        apply_mask(
            "mouse_buttons",
            self.x11_connection.clone(),
            self.input_mask.clone(),
        )
        .await;
    }

    async fn on_stop(&self) {
        self.input_mask
            .remove(XIEventMask::RAW_BUTTON_PRESS | XIEventMask::RAW_BUTTON_RELEASE);

        apply_mask(
            "mouse_buttons",
            self.x11_connection.clone(),
            self.input_mask.clone(),
        )
        .await;
    }
}

#[derive(Debug, Constructor)]
pub struct MouseMoveTopic {
    x11_connection: Arc<X11Connection>,
    input_mask: Arc<InputMask>,
}

impl Topic for MouseMoveTopic {
    type T = Point;
    type Signal = LatestOnlySignals<Self::T>;

    async fn on_start(&self) {
        self.input_mask.set(XIEventMask::MOTION);

        apply_mask(
            "mouse_buttons",
            self.x11_connection.clone(),
            self.input_mask.clone(),
        )
        .await;
    }

    async fn on_stop(&self) {
        self.input_mask.remove(XIEventMask::MOTION);

        apply_mask(
            "mouse_buttons",
            self.x11_connection.clone(),
            self.input_mask.clone(),
        )
        .await;
    }
}
