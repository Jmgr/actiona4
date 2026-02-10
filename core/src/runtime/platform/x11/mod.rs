use std::{
    fmt::Debug,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use color_eyre::{Result, eyre::eyre};
use derive_more::{Deref, DerefMut};
use libwmctl::AtomCollection;
use tokio::{runtime::Builder, select, sync::broadcast, task::LocalSet, time::sleep};
use tokio_util::{
    sync::{CancellationToken, DropGuard},
    task::TaskTracker,
};
use tracing::{error, info, instrument};
use x11rb::protocol::xinput::{Device, DeviceType, EventMask, XIEventMask};
use x11rb_async::{
    connection::Connection,
    protocol::{
        Event,
        randr::{self},
        shm,
        xinput::{xi_query_version, xi_select_events},
        xkb::{
            BoolCtrl, ConnectionExt, ControlsNotifyEvent, EventType, MapPart, SelectEventsAux,
            get_controls,
        },
        xproto::get_keyboard_control,
    },
};
use xkbcommon::xkb::{
    self, CONTEXT_NO_FLAGS, KeyDirection,
    x11::{
        MIN_MAJOR_XKB_VERSION, MIN_MINOR_XKB_VERSION, SetupXkbExtensionFlags,
        get_core_keyboard_device_id, setup_xkb_extension,
    },
};
use xkeysym::{KeyCode, Keysym};

use crate::{
    api::{
        displays::Displays, mouse::Button, point::point,
        windows::platform::x11::events::WindowEvent,
    },
    platform::x11::X11Connection,
    runtime::{
        events::{Guard, KeyboardKeyEvent, KeyboardTextEvent, MouseButtonEvent, TopicWrapper},
        platform::x11::events::input::{
            ActivationCounter, InputMask, KeyboardKeysTopic, KeyboardTextTopic, MouseButtonsTopic,
            MouseMoveTopic, keysym_to_key,
        },
    },
    types::input::Direction,
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
    mouse_buttons_topic: TopicWrapper<MouseButtonsTopic>,
    mouse_move_topic: TopicWrapper<MouseMoveTopic>,
    keyboard_keys_topic: TopicWrapper<KeyboardKeysTopic>,
    keyboard_text_topic: TopicWrapper<KeyboardTextTopic>,
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
    #[instrument(name = "X11Runtime::new", skip_all)]
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        display_name: Option<&str>,
        displays: Displays,
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

        {
            let root_window = x11_connection.screen().root;
            xi_select_events(
                connection,
                root_window,
                &[EventMask {
                    deviceid: Device::ALL.into(),
                    mask: vec![XIEventMask::HIERARCHY],
                }],
            )
            .await?;
        }

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
        let reply = connection
            .xkb_use_extension(MIN_MAJOR_XKB_VERSION, MIN_MINOR_XKB_VERSION)
            .await?
            .reply()
            .await?;
        if !reply.supported {
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

        let core_keyboard = get_core_keyboard_device_id(xcb_connection).try_into()?;
        let map_parts =
            MapPart::KEY_TYPES | MapPart::KEY_SYMS | MapPart::MODIFIER_MAP | MapPart::VIRTUAL_MODS;

        connection
            .xkb_select_events(
                core_keyboard,
                EventType::default(),
                EventType::NEW_KEYBOARD_NOTIFY | EventType::MAP_NOTIFY | EventType::CONTROLS_NOTIFY,
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

        let input_mask = InputMask::new(x11_connection.clone()).await?;
        let mouse_buttons_topic = TopicWrapper::new(
            MouseButtonsTopic::new(input_mask.clone()),
            cancellation_token.clone(),
            task_tracker.clone(),
        );
        let mouse_move_topic = TopicWrapper::new(
            MouseMoveTopic::new(input_mask.clone()),
            cancellation_token.clone(),
            task_tracker.clone(),
        );
        let activation_counter = ActivationCounter::default();
        let keyboard_keys_topic = TopicWrapper::new(
            KeyboardKeysTopic::new(input_mask.clone(), activation_counter.clone()),
            cancellation_token.clone(),
            task_tracker.clone(),
        );
        let keyboard_text_topic = TopicWrapper::new(
            KeyboardTextTopic::new(input_mask, activation_counter),
            cancellation_token.clone(),
            task_tracker.clone(),
        );
        let (window_event_sender, _) = broadcast::channel(1024);

        let local_cancellation_token = cancellation_token.clone();
        let local_x11_connection = x11_connection.clone();
        let local_mouse_buttons_topic = mouse_buttons_topic.clone();
        let local_mouse_move_topic = mouse_move_topic.clone();
        let local_keyboard_keys_topic = keyboard_keys_topic.clone();
        let local_keyboard_text_topic = keyboard_text_topic.clone();
        let local_window_event_sender = window_event_sender.clone();

        let main_loop_thread = spawn_on_dedicated_thread(move || async move {
            let mut input_devices = InputDevices::new(local_x11_connection.clone()).await?;
            let mut keyboard_state = KeyboardState::new(local_x11_connection.clone());

            let connection = local_x11_connection.async_connection();

            let mut key_repeat =
                KeyRepeat::new(local_x11_connection.clone(), core_keyboard).await?;
            let mut repeating_key = None;

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
                    Event::Error(event) => {
                        eprintln!("X11Error: {event:?}");
                    }
                    Event::XinputHierarchy(_event) => {
                        input_devices.refresh().await?;
                    }
                    Event::XinputRawButtonPress(event) => {
                        let button = Button::from_event(event.detail)?;
                        local_mouse_buttons_topic.publish(MouseButtonEvent {
                            button,
                            direction: Direction::Press,
                            is_injected: input_devices.is_xtest_pointer(event.sourceid),
                        });
                    }
                    Event::XinputRawButtonRelease(event) => {
                        let button = Button::from_event(event.detail)?;
                        local_mouse_buttons_topic.publish(MouseButtonEvent {
                            button,
                            direction: Direction::Release,
                            is_injected: input_devices.is_xtest_pointer(event.sourceid),
                        });
                    }
                    Event::XinputMotion(event) => {
                        local_mouse_move_topic.publish(point(event.root_x, event.root_y)); // TODO: injected?
                    }
                    Event::XinputRawKeyPress(event) => {
                        let keycode = event.detail.into();
                        let keysym = keyboard_state.key_get_one_sym(keycode);
                        let name = keyboard_state.key_get_utf8(keycode);

                        keyboard_state.update_key(keycode, KeyDirection::Down);

                        let text_event = if !matches!(
                            keysym,
                            Keysym::BackSpace | Keysym::Return | Keysym::KP_Enter // TODO: hackish
                        ) {
                            name.chars().next().map(|char| KeyboardTextEvent {
                                character: char,
                                is_injected: input_devices.is_xtest_keyboard(event.sourceid),
                                is_repeat: false,
                            }) // TODO: use graphemes?
                        } else {
                            None
                        };

                        if let Some(ref text_event) = text_event {
                            local_keyboard_text_topic.publish(text_event.clone());
                        }

                        let key_event = KeyboardKeyEvent {
                            key: keysym_to_key(keysym),
                            scan_code: keycode.into(),
                            direction: Direction::Press,
                            is_injected: input_devices.is_xtest_keyboard(event.sourceid),
                            name,
                            is_repeat: false,
                        };
                        local_keyboard_keys_topic.publish(key_event.clone());

                        if key_repeat.is_enabled() && key_repeat.should_key_code_repeat(keycode)? {
                            let cancellation_token = local_cancellation_token.child_token();
                            let delay = key_repeat.repeat_delay();
                            let interval = key_repeat.repeat_interval();
                            let local_keyboard_keys_topic = local_keyboard_keys_topic.clone();
                            let local_keyboard_text_topic = local_keyboard_text_topic.clone();

                            let mut key_event = key_event;
                            key_event.is_repeat = true;
                            let text_event = text_event.map(|mut event| {
                                event.is_repeat = true;
                                event
                            });

                            repeating_key = Some(RepeatingKey {
                                key: keycode,
                                drop_guard: cancellation_token.clone().drop_guard(),
                            });

                            task_tracker.spawn(async move {
                                select! {
                                    _ = sleep(Duration::from_millis(delay.into())) => {},
                                    _ = cancellation_token.cancelled() => { return; },
                                }

                                loop {
                                    local_keyboard_keys_topic.publish(key_event.clone());
                                    if let Some(ref text_event) = text_event {
                                        local_keyboard_text_topic.publish(text_event.clone());
                                    }

                                    select! {
                                        _ = sleep(Duration::from_millis(interval.into())) => {},
                                        _ = cancellation_token.cancelled() => { return; },
                                    }
                                }
                            });
                        }
                    }
                    Event::XinputRawKeyRelease(event) => {
                        let keycode = event.detail.into();
                        let keysym = keyboard_state.key_get_one_sym(keycode);
                        let name = keyboard_state.key_get_utf8(keycode);

                        keyboard_state.update_key(keycode, KeyDirection::Up);

                        // Stop the key repeat task if it's the same key
                        repeating_key.take_if(|repeating_key| repeating_key.key == keycode);

                        local_keyboard_keys_topic.publish(KeyboardKeyEvent {
                            key: keysym_to_key(keysym),
                            scan_code: keycode.into(),
                            direction: Direction::Release,
                            is_injected: input_devices.is_xtest_keyboard(event.sourceid),
                            name,
                            is_repeat: false,
                        });
                    }
                    Event::XkbControlsNotify(event) => {
                        key_repeat.refresh_from_event(&event).await?;

                        // Stop all key repeats
                        if !key_repeat.is_enabled() {
                            repeating_key.take();
                        }
                    }
                    Event::XkbNewKeyboardNotify(_) => {
                        key_repeat.refresh().await?;

                        // Stop all key repeats
                        if !key_repeat.is_enabled() {
                            repeating_key.take();
                        }

                        keyboard_state = KeyboardState::new(local_x11_connection.clone());
                    }
                    Event::XkbMapNotify(_) => {
                        keyboard_state = KeyboardState::new(local_x11_connection.clone());
                    }
                    Event::RandrScreenChangeNotify(_event) => {
                        displays.refresh();
                    }
                    Event::DestroyNotify(e) => {
                        let handle = libwmctl::window(e.window).into();
                        _ = local_window_event_sender.send(WindowEvent::Closed(handle));
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
            keyboard_text_topic,
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
    pub fn mouse_buttons(&self) -> Guard<MouseButtonsTopic> {
        self.mouse_buttons_topic.subscribe()
    }

    #[must_use]
    pub fn mouse_move(&self) -> Guard<MouseMoveTopic> {
        self.mouse_move_topic.subscribe()
    }

    #[must_use]
    pub fn keyboard_keys(&self) -> Guard<KeyboardKeysTopic> {
        self.keyboard_keys_topic.subscribe()
    }

    #[must_use]
    pub fn keyboard_text(&self) -> Guard<KeyboardTextTopic> {
        self.keyboard_text_topic.subscribe()
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

struct RepeatingKey {
    key: KeyCode,
    #[allow(unused)]
    drop_guard: DropGuard,
}

struct KeyRepeat {
    x11_connection: Arc<X11Connection>,
    core_keyboard: u16,
    is_enabled: bool,
    repeat_delay: u16,
    repeat_interval: u16,
    auto_repeat_keys: [bool; 256],
}

impl KeyRepeat {
    pub async fn new(x11_connection: Arc<X11Connection>, core_keyboard: u16) -> Result<Self> {
        let (is_enabled, repeat_delay, repeat_interval, auto_repeat_keys) =
            Self::fetch(x11_connection.clone(), core_keyboard).await?;

        Ok(Self {
            x11_connection,
            core_keyboard,
            is_enabled,
            repeat_delay,
            repeat_interval,
            auto_repeat_keys,
        })
    }

    pub async fn refresh_from_event(&mut self, event: &ControlsNotifyEvent) -> Result<()> {
        let is_enabled = event.enabled_controls.contains(BoolCtrl::REPEAT_KEYS);
        if is_enabled {
            self.refresh().await?;
        }
        self.is_enabled = is_enabled;
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let (is_enabled, repeat_delay, repeat_interval, auto_repeat_keys) =
            Self::fetch(self.x11_connection.clone(), self.core_keyboard).await?;

        self.is_enabled = is_enabled;
        self.repeat_delay = repeat_delay;
        self.repeat_interval = repeat_interval;
        self.auto_repeat_keys = auto_repeat_keys;

        Ok(())
    }

    pub const fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub const fn repeat_delay(&self) -> u16 {
        self.repeat_delay
    }

    pub const fn repeat_interval(&self) -> u16 {
        self.repeat_interval
    }

    pub fn should_key_code_repeat(&self, key_code: KeyCode) -> Result<bool> {
        let result = self
            .auto_repeat_keys
            .get::<usize>(key_code.raw().try_into()?)
            .ok_or_else(|| eyre!("unexpected key code: {key_code:?}"))?;
        Ok(*result)
    }

    async fn fetch(
        x11_connection: Arc<X11Connection>,
        core_keyboard: u16,
    ) -> Result<(bool, u16, u16, [bool; 256])> {
        let connection = x11_connection.async_connection();
        let controls = get_controls(connection, core_keyboard)
            .await?
            .reply()
            .await?;
        let keyboard_control = get_keyboard_control(connection).await?.reply().await?;
        let mut auto_repeat_keys = [false; 256];
        for (i, byte) in keyboard_control.auto_repeats.iter().copied().enumerate() {
            for bit in 0..8 {
                let key_code = u8::try_from(i * 8 + bit)?;
                auto_repeat_keys[usize::from(key_code)] = (byte & (1 << bit)) != 0;
            }
        }

        Ok((
            controls.enabled_controls.contains(BoolCtrl::REPEAT_KEYS),
            controls.repeat_delay,
            controls.repeat_interval,
            auto_repeat_keys,
        ))
    }
}

struct InputDevices {
    x11_connection: Arc<X11Connection>,
    xtest_keyboard: u16,
    xtest_pointer: u16,
}

impl InputDevices {
    pub async fn new(x11_connection: Arc<X11Connection>) -> Result<Self> {
        let (xtest_keyboard, xtest_pointer) = Self::refresh_impl(x11_connection.clone()).await?;

        Ok(Self {
            x11_connection,
            xtest_keyboard,
            xtest_pointer,
        })
    }

    pub const fn is_xtest_keyboard(&self, deviceid: u16) -> bool {
        self.xtest_keyboard == deviceid
    }

    pub const fn is_xtest_pointer(&self, deviceid: u16) -> bool {
        self.xtest_pointer == deviceid
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let (xtest_keyboard, xtest_pointer) =
            Self::refresh_impl(self.x11_connection.clone()).await?;

        self.xtest_keyboard = xtest_keyboard;
        self.xtest_pointer = xtest_pointer;

        Ok(())
    }

    async fn refresh_impl(x11_connection: Arc<X11Connection>) -> Result<(u16, u16)> {
        use x11rb_async::protocol::xinput::ConnectionExt;

        let connection = x11_connection.async_connection();
        let devices = connection
            .xinput_xi_query_device(Device::ALL)
            .await?
            .reply()
            .await?;

        let mut xtest_keyboard = None;
        let mut xtest_pointer = None;

        for device in devices.infos {
            let name = String::from_utf8_lossy(&device.name);
            if !name.contains("XTEST") {
                continue;
            }

            match device.type_ {
                DeviceType::SLAVE_KEYBOARD => xtest_keyboard = Some(device.deviceid),
                DeviceType::SLAVE_POINTER => xtest_pointer = Some(device.deviceid),
                _ => {}
            }
        }

        Ok((
            xtest_keyboard.ok_or_else(|| eyre!("failed to find the XTest keyboard"))?,
            xtest_pointer.ok_or_else(|| eyre!("failed to find the XTest pointer"))?,
        ))
    }
}
