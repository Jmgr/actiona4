use std::{
    fmt::Debug,
    sync::Arc,
    thread::{self, JoinHandle},
};

use derive_more::{Deref, DerefMut};
use enigo::Direction;
use eyre::{Result, eyre};
use libwmctl::AtomCollection;
use tokio::{runtime::Builder, select, sync::broadcast, task::LocalSet};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use x11rb::protocol::xinput::PointerEventFlags;
use x11rb_async::{
    connection::Connection,
    protocol::{
        Event,
        randr::{self},
        shm,
        xinput::xi_query_version,
        xkb::{ConnectionExt, EventType, MapPart, SelectEventsAux},
    },
};
use xkbcommon::xkb::{
    self, CONTEXT_NO_FLAGS, KeyDirection,
    x11::{
        MIN_MAJOR_XKB_VERSION, MIN_MINOR_XKB_VERSION, SetupXkbExtensionFlags,
        get_core_keyboard_device_id, setup_xkb_extension,
    },
};

use crate::{
    core::{mouse::Button, point::point, windows::platform::x11::events::WindowEvent},
    platform::x11::X11Connection,
    runtime::{
        events::{KeyboardKeyEvent, MouseButtonEvent, TopicWrapper},
        platform::x11::events::{
            displays::ScreenChangeTopic,
            input::{
                InputMask, KeyboardKeysTopic, MouseButtonsTopic, MouseMoveTopic, keysym_to_key,
            },
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
    keyboard_keys_topic: Arc<TopicWrapper<KeyboardKeysTopic>>,
    screen_change_topic: Arc<TopicWrapper<ScreenChangeTopic>>,
    window_event_sender: broadcast::Sender<WindowEvent>,
    main_loop_thread: Option<JoinHandle<Result<()>>>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if let Some(main_loop_thread) = self.main_loop_thread.take()
            && let Err(err) = main_loop_thread.join().expect("thread join should succeed")
        {
            error!("main loop thread finished with an error: {err}");
        }
    }
}

impl Runtime {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        display_name: Option<&str>,
    ) -> Result<Self> {
        let x11_connection = Arc::new(
            X11Connection::new(
                cancellation_token.clone(),
                task_tracker.clone(),
                display_name,
            )
            .await?,
        );
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

        // Use XKB
        let ue = connection
            .xkb_use_extension(MIN_MAJOR_XKB_VERSION, MIN_MINOR_XKB_VERSION)
            .await?
            .reply()
            .await?;
        if !ue.supported {
            return Err(eyre!("required XKB extension not available"));
        }

        let xcb_connection = x11_connection.xcb_connection();
        let mut major_version = 0;
        let mut minor_version = 0;
        let mut base_event = 0;
        let mut base_error = 0;
        if !setup_xkb_extension(
            xcb_connection,
            MIN_MAJOR_XKB_VERSION,
            MIN_MINOR_XKB_VERSION,
            SetupXkbExtensionFlags::NoFlags,
            &mut major_version,
            &mut minor_version,
            &mut base_event,
            &mut base_error,
        ) {
            return Err(eyre!("required XKB extension not available"));
        }
        info!(
            "XKB available, version: {}.{}",
            major_version, minor_version
        );

        let map_parts =
            MapPart::KEY_TYPES | MapPart::KEY_SYMS | MapPart::MODIFIER_MAP | MapPart::VIRTUAL_MODS;

        connection
            .xkb_select_events(
                get_core_keyboard_device_id(xcb_connection).try_into()?,
                EventType::default(),
                EventType::NEW_KEYBOARD_NOTIFY | EventType::MAP_NOTIFY,
                map_parts,
                map_parts,
                &SelectEventsAux::new(),
            )
            .await?;
        connection.flush().await?;

        // Check if SHM is available (used for faster screenshots)
        let has_shm = async {
            let version = shm::query_version(connection).await?.reply().await?;
            Result::<shm::QueryVersionReply>::Ok(version)
        }
        .await
        .map_or_else(
            |_| {
                info!("Shm not available");

                false
            },
            |version| {
                info!(
                    "Shm available, version: {}.{}",
                    version.major_version, version.minor_version
                );

                true
            },
        );

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
        let keyboard_keys_topic = Arc::new(TopicWrapper::new(
            KeyboardKeysTopic::new(x11_connection.clone(), input_mask),
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
        let local_keyboard_keys_topic = keyboard_keys_topic.clone();
        let local_screen_change_topic = screen_change_topic.clone();
        let local_window_event_sender = window_event_sender.clone();

        let main_loop_thread = spawn_on_dedicated_thread(|| async move {
            let mut keyboard_state = KeyboardState::new(local_x11_connection.clone());

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

                match event {
                    Event::XinputRawButtonPress(event) => {
                        let button = Button::from_event(event.detail)?;
                        local_mouse_buttons_topic.publish(MouseButtonEvent {
                            button,
                            direction: Direction::Press,
                            injected: event.flags == PointerEventFlags::POINTER_EMULATED,
                        });
                    }
                    Event::XinputRawButtonRelease(event) => {
                        let button = Button::from_event(event.detail)?;
                        local_mouse_buttons_topic.publish(MouseButtonEvent {
                            button,
                            direction: Direction::Release,
                            injected: event.flags == PointerEventFlags::POINTER_EMULATED,
                        });
                    }
                    Event::XinputMotion(event) => {
                        local_mouse_move_topic.publish(point(event.root_x, event.root_y));
                    }
                    Event::XinputRawKeyPress(event) => {
                        let keycode = event.detail.into();
                        let keysym = keyboard_state.key_get_one_sym(keycode);
                        let name = keyboard_state.key_get_utf8(keycode);

                        keyboard_state.update_key(keycode, KeyDirection::Down);

                        local_keyboard_keys_topic.publish(KeyboardKeyEvent {
                            key: keysym_to_key(keysym),
                            direction: Direction::Press,
                            injected: false, // Unsupported
                            name,
                        });
                    }
                    Event::XinputRawKeyRelease(event) => {
                        let keycode = event.detail.into();
                        let keysym = keyboard_state.key_get_one_sym(keycode);
                        let name = keyboard_state.key_get_utf8(keycode);

                        keyboard_state.update_key(keycode, KeyDirection::Up);

                        local_keyboard_keys_topic.publish(KeyboardKeyEvent {
                            key: keysym_to_key(keysym),
                            direction: Direction::Release,
                            injected: false, // Unsupported
                            name,
                        });
                    }
                    Event::XkbMapNotify(_) | Event::XkbNewKeyboardNotify(_) => {
                        keyboard_state = KeyboardState::new(local_x11_connection.clone());
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
                    _ => {}
                };
            }

            Result::<()>::Ok(())
        });

        Ok(Self {
            x11_connection,
            has_shm,
            atoms,
            mouse_buttons_topic,
            mouse_move_topic,
            keyboard_keys_topic,
            screen_change_topic,
            window_event_sender,
            main_loop_thread: Some(main_loop_thread),
        })
    }

    #[must_use]
    pub fn x11_connection(&self) -> Arc<X11Connection> {
        self.x11_connection.clone()
    }

    #[must_use]
    pub const fn has_shm(&self) -> bool {
        self.has_shm
    }

    #[must_use]
    pub const fn atoms(&self) -> &AtomCollection {
        &self.atoms
    }

    #[must_use]
    pub fn mouse_buttons(&self) -> Arc<TopicWrapper<MouseButtonsTopic>> {
        self.mouse_buttons_topic.clone()
    }

    #[must_use]
    pub fn mouse_move(&self) -> Arc<TopicWrapper<MouseMoveTopic>> {
        self.mouse_move_topic.clone()
    }

    #[must_use]
    pub fn keyboard_keys(&self) -> Arc<TopicWrapper<KeyboardKeysTopic>> {
        self.keyboard_keys_topic.clone()
    }

    #[must_use]
    pub fn screen_change(&self) -> Arc<TopicWrapper<ScreenChangeTopic>> {
        self.screen_change_topic.clone() // TODO: return guard?
    }

    #[must_use]
    pub fn subscribe_window_events(&self) -> broadcast::Receiver<WindowEvent> {
        self.window_event_sender.subscribe()
    }
}

fn spawn_on_dedicated_thread<F, Fut, T>(make_fut: F) -> JoinHandle<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = T> + 'static,
    T: Debug + Send + 'static,
{
    thread::spawn(move || {
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let local_set = LocalSet::new();

        local_set.block_on(&runtime, async move { Box::pin((make_fut)()).await })
    })
}

#[derive(Deref, DerefMut)]
struct KeyboardState(xkb::State);

impl KeyboardState {
    pub fn new(x11_connection: Arc<X11Connection>) -> Self {
        let xcb_connection = x11_connection.xcb_connection();
        let ctx = xkb::Context::new(CONTEXT_NO_FLAGS);
        let dev = xkb::x11::get_core_keyboard_device_id(xcb_connection);
        let keymap = xkb::x11::keymap_new_from_device(
            &ctx,
            xcb_connection,
            dev,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );

        Self(xkb::x11::state_new_from_device(
            &keymap,
            xcb_connection,
            dev,
        ))
    }
}
