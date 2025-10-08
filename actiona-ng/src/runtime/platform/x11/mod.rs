use std::sync::Arc;

use enigo::Direction;
use eyre::{Result, eyre};
use libwmctl::AtomCollection;
use tokio::{select, sync::broadcast};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use x11rb_async::{
    connection::Connection,
    protocol::{
        Event,
        randr::{self},
        shm,
        xinput::xi_query_version,
    },
};

use crate::{
    core::{mouse::Button, point::point, windows::platform::x11::events::WindowEvent},
    platform::x11::X11Connection,
    runtime::{
        events::{MouseButtonEvent, TopicWrapper},
        platform::x11::events::{
            displays::ScreenChangeTopic,
            input::{InputMask, MouseButtonsTopic, MouseMoveTopic},
        },
    },
};

pub mod events;

impl Button {
    fn from_event(detail: u32) -> Result<Self> {
        Ok(match detail {
            1 => Self::Left,
            2 => Self::Middle,
            3 => Self::Right,
            8 => Self::Back,
            9 => Self::Forward,
            _ => return Err(eyre!("unknown button event detail: {detail}")),
        })
    }
}

#[derive(Debug)]
pub struct Runtime {
    x11_connection: Arc<X11Connection>,
    has_shm: bool,
    atoms: AtomCollection,
    mouse_buttons_topic: Arc<TopicWrapper<MouseButtonsTopic>>,
    mouse_move_topic: Arc<TopicWrapper<MouseMoveTopic>>,
    screen_change_topic: Arc<TopicWrapper<ScreenChangeTopic>>,
    window_event_sender: broadcast::Sender<WindowEvent>,
}

impl Runtime {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Self> {
        let x11_connection =
            Arc::new(X11Connection::new(cancellation_token.clone(), task_tracker.clone()).await?);
        let connection = x11_connection.async_connection();
        let atoms = AtomCollection::new(x11_connection.sync_connection())?.reply()?;

        // Make sure XInput2 is available
        let version = xi_query_version(connection, 2, 4).await?.reply().await?;
        info!(
            "XInput2 available, version: {}.{}",
            version.major_version, version.minor_version
        );

        // Make sure RandR is available
        let version = randr::query_version(connection, 1, 6)
            .await?
            .reply()
            .await?;
        info!(
            "RandR available, version: {}.{}",
            version.major_version, version.minor_version
        );

        let has_shm = if let Ok(version) = async {
            let version = shm::query_version(connection).await?.reply().await?;
            Result::<shm::QueryVersionReply>::Ok(version)
        }
        .await
        {
            info!(
                "Shm available, version: {}.{}",
                version.major_version, version.minor_version
            );

            true
        } else {
            info!("Shm not available");

            false
        };

        let input_mask = Arc::new(InputMask::default());
        let mouse_buttons_topic = Arc::new(TopicWrapper::new(
            MouseButtonsTopic::new(x11_connection.clone(), input_mask.clone()),
            cancellation_token.clone(),
            task_tracker.clone(),
        ));
        let mouse_move_topic = Arc::new(TopicWrapper::new(
            MouseMoveTopic::new(x11_connection.clone(), input_mask.clone()),
            cancellation_token.clone(),
            task_tracker.clone(),
        ));
        let screen_change_topic = Arc::new(TopicWrapper::new(
            ScreenChangeTopic::new(x11_connection.clone()),
            cancellation_token.clone(),
            task_tracker.clone(),
        ));
        let (window_event_sender, _) = broadcast::channel(1024);

        let local_cancellation_token = cancellation_token.clone();
        let local_x11_connection = x11_connection.clone();
        let local_mouse_buttons_topic = mouse_buttons_topic.clone();
        let local_mouse_move_topic = mouse_move_topic.clone();
        let local_screen_change_topic = screen_change_topic.clone();
        let local_window_event_sender = window_event_sender.clone();

        task_tracker.spawn(async move {
            let connection = local_x11_connection.async_connection();
            loop {
                let event = select! {
                    _ = local_cancellation_token.cancelled() => { break; }
                    event = connection
                    .wait_for_event() => {
                        let Ok(event) = event else {
                            break;
                        };

                       event
                    }
                };

                if let Err(err) = (|| {
                    Result::<()>::Ok(match event {
                        Event::XinputRawButtonPress(event) => {
                            let button = Button::from_event(event.detail)?;
                            local_mouse_buttons_topic.publish(MouseButtonEvent {
                                button,
                                direction: Direction::Press,
                            });
                        }
                        Event::XinputRawButtonRelease(event) => {
                            let button = Button::from_event(event.detail)?;
                            local_mouse_buttons_topic.publish(MouseButtonEvent {
                                button,
                                direction: Direction::Release,
                            });
                        }
                        Event::XinputMotion(event) => {
                            local_mouse_move_topic.publish(point(event.root_x, event.root_y));
                        }
                        Event::RandrScreenChangeNotify(_event) => {
                            match display_info::DisplayInfo::all() {
                                Ok(infos) => {
                                    local_screen_change_topic.publish(infos.into());
                                }
                                Err(err) => {
                                    error!("fetching display info: {err}");
                                }
                            }
                        }
                        Event::DestroyNotify(e) => {
                            let handle = libwmctl::window(e.window).into();
                            let _ = local_window_event_sender.send(WindowEvent::Closed(handle));
                        }

                        /*
                        Event::XinputRawKeyPress(event) => Button::from_event(event.detail)
                            .map(|button| RecordEvent::MouseButton(button, Direction::Pressed)), // TODO: keys
                        Event::XinputRawKeyRelease(event) => Button::from_event(event.detail)
                            .map(|button| RecordEvent::MouseButton(button, Direction::Released)), // TODO: keys

                        */
                        _ => {}
                    })
                })() {
                    error!("x11 event: {err}");
                };
            }
        });

        Ok(Self {
            x11_connection,
            has_shm,
            atoms,
            mouse_buttons_topic,
            mouse_move_topic,
            screen_change_topic,
            window_event_sender,
        })
    }

    pub fn x11_connection(&self) -> Arc<X11Connection> {
        self.x11_connection.clone()
    }

    pub const fn has_shm(&self) -> bool {
        self.has_shm
    }

    pub fn atoms(&self) -> &AtomCollection {
        &self.atoms
    }

    pub fn mouse_buttons(&self) -> Arc<TopicWrapper<MouseButtonsTopic>> {
        self.mouse_buttons_topic.clone()
    }

    pub fn mouse_move(&self) -> Arc<TopicWrapper<MouseMoveTopic>> {
        self.mouse_move_topic.clone()
    }

    pub fn screen_change(&self) -> Arc<TopicWrapper<ScreenChangeTopic>> {
        self.screen_change_topic.clone() // TODO: return guard?
    }

    pub fn subscribe_window_events(&self) -> broadcast::Receiver<WindowEvent> {
        self.window_event_sender.subscribe()
    }
}
