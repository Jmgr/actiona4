use std::sync::Arc;

use eyre::Result;
use tokio::{select, sync::broadcast::Sender};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use x11rb_async::{
    connection::Connection,
    protocol::{
        Event,
        randr::{self, ConnectionExt},
        shm,
        xinput::{Device, EventMask, xi_query_version, xi_select_events},
    },
};

use crate::{
    core::{displays, keyboard, mouse, mouse::JsButton},
    platform::x11::X11Connection,
    runtime::{Direction, RecordEvent},
};

impl JsButton {
    const fn from_event(detail: u32) -> Option<Self> {
        Some(match detail {
            1 => Self::Left,
            2 => Self::Middle,
            3 => Self::Right,
            8 => Self::Back,
            9 => Self::Forward,
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub struct Runtime {
    x11_connection: Arc<X11Connection>,
    has_shm: bool,
}

impl Runtime {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        events_sender: Sender<RecordEvent>,
    ) -> Result<Self> {
        let x11_connection =
            Arc::new(X11Connection::new(cancellation_token.clone(), task_tracker.clone()).await?);
        let connection = x11_connection.connection();

        // Make sure XInput2 is available
        xi_query_version(connection, 2, 4).await?.reply().await?;

        // Make sure RandR is available
        randr::query_version(connection, 1, 6)
            .await?
            .reply()
            .await?;

        let has_shm = async {
            shm::query_version(connection).await?.reply().await?;
            Result::<()>::Ok(())
        }
        .await
        .is_ok();

        let randr_event_mask = displays::platform::x11::DisplaysImpl::randr_event_mask();

        x11_connection
            .connection()
            .randr_select_input(x11_connection.screen().root, randr_event_mask)
            .await?;

        let xinput_event_mask = mouse::platform::x11::MouseImpl::xinput_event_mask()
            | keyboard::platform::x11::KeyboardImpl::xinput_event_mask();

        xi_select_events(
            connection,
            x11_connection.screen().root,
            &[EventMask {
                deviceid: Device::ALL_MASTER.into(),
                mask: vec![xinput_event_mask],
            }],
        )
        .await?;

        // This is needed to receive events
        connection.flush().await?;

        let local_cancellation_token = cancellation_token.clone();
        let local_x11_connection = x11_connection.clone();
        let local_events_sender = events_sender.clone();

        task_tracker.spawn(async move {
            loop {
                select! {
                    _ = local_cancellation_token.cancelled() => { break; }
                    event = local_x11_connection
                    .connection()
                    .wait_for_event() => {
                        let Ok(event) = event else {
                            break;
                        };

                        Self::process_event(event, &local_events_sender);
                    }
                }
            }
        });

        Ok(Self {
            x11_connection,
            has_shm,
        })
    }

    fn process_event(event: Event, sender: &Sender<RecordEvent>) {
        let event = match event {
            Event::XinputRawButtonPress(event) => JsButton::from_event(event.detail)
                .map(|button| RecordEvent::MouseButton(button, Direction::Pressed)),
            Event::XinputRawButtonRelease(event) => JsButton::from_event(event.detail)
                .map(|button| RecordEvent::MouseButton(button, Direction::Released)),
            Event::XinputRawKeyPress(event) => JsButton::from_event(event.detail)
                .map(|button| RecordEvent::MouseButton(button, Direction::Pressed)), // TODO: keys
            Event::XinputRawKeyRelease(event) => JsButton::from_event(event.detail)
                .map(|button| RecordEvent::MouseButton(button, Direction::Released)), // TODO: keys
            Event::RandrScreenChangeNotify(_event) => {
                let infos = display_info::DisplayInfo::all().unwrap();
                Some(RecordEvent::DisplayChanged(infos.into()))
            }
            _ => None,
        };

        if let Some(event) = event {
            sender.send(event).unwrap();
        }
    }

    pub fn x11_connection(&self) -> Arc<X11Connection> {
        self.x11_connection.clone()
    }

    pub const fn has_shm(&self) -> bool {
        self.has_shm
    }
}
