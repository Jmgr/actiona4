#[cfg(test)]
use std::sync::OnceLock;
#[cfg(windows)]
#[cfg_attr(test, allow(unused_imports))]
use std::sync::atomic::AtomicBool;
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
};

use color_eyre::Result;
use derive_more::Constructor;
use derive_where::derive_where;
use enigo::{Enigo, Settings};
#[cfg(unix)]
#[cfg_attr(test, allow(unused_imports))]
use ksni::TrayMethods as _;
use macros::{FromSerde, IntoSerde};
use opencv::core::set_num_threads;
use parking_lot::Mutex;
use rquickjs::{Ctx, JsLifetime, runtime::UserDataGuard};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumIter, FromRepr};
use tokio::{runtime::Handle, select, signal, task::block_in_place};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument, warn};
#[cfg(windows)]
#[cfg_attr(test, allow(unused_imports))]
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
#[cfg(windows)]
#[cfg_attr(test, allow(unused_imports))]
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::run_on_demand::EventLoopExtRunOnDemand,
    window::WindowId,
};

#[cfg(unix)]
use crate::runtime::platform::x11::events::input::{
    KeyboardKeysTopic, KeyboardTextTopic, MouseButtonsTopic, MouseMoveTopic, MouseScrollTopic,
};
#[cfg(windows)]
use crate::runtime::win::events::input::{
    keyboard::KeyboardKeysTopic, keyboard::KeyboardTextTopic, mouse::MouseButtonsTopic,
    mouse::MouseMoveTopic, mouse::MouseScrollTopic,
};
use crate::{
    IntoJSError,
    api::{
        app::js::JsApp,
        audio::{PlayingSoundsTracker, js::JsAudio},
        clipboard::{Clipboard, js::JsClipboard},
        color::js::JsColor,
        console::js::JsConsole,
        datetime::js::JsDatetime,
        dialogs::js::JsDialogs,
        directory::js::JsDirectory,
        displays::{Displays, js::JsDisplays},
        file::js::JsFile,
        filesystem::js::JsFilesystem,
        image::{
            find_image,
            js::{JsImage, JsMatch},
        },
        js::{
            abort_controller::{JsAbortController, JsAbortSignal},
            classes::{register_host_class, register_singleton_class, register_value_class},
            concurrency::JsConcurrency,
            global,
        },
        keyboard::{Keyboard, js::JsKeyboard},
        macros::{js::JsMacros, player::MacroPlayer},
        mouse::{Mouse, js::JsMouse},
        name::js::JsWildcard,
        notification::js::JsNotification,
        path::js::JsPath,
        point::js::JsPoint,
        process::js::JsProcess,
        random::js::JsRandom,
        rect::js::JsRect,
        screen::{Screen, js::JsScreen},
        size::js::JsSize,
        standardpaths::js::JsStandardPaths,
        system::js::JsSystem,
        web::js::JsWeb,
        windows::{Windows, js::JsWindows},
    },
    cancel_on,
    error::CommonError,
    platform_info::{Platform, is_linux},
    runtime::{events::Guard, extensions::Extensions, shared_rng::SharedRng},
    scripting::{Engine as ScriptEngine, UnhandledException, callbacks::Callbacks},
};

pub mod async_resource;
pub mod events;
pub mod extensions;
pub mod platform;
pub mod shared_rng;

#[cfg(windows)]
use platform::win;
#[cfg(unix)]
use platform::x11;

#[cfg(unix)]
pub fn ensure_x11_session_available(display_name: Option<&str>) -> Result<()> {
    crate::platform::x11::ensure_session_available(display_name)
}

pub(crate) trait WithUserData {
    fn user_data<'a>(&'a self) -> UserDataGuard<'a, JsUserData>;
}

impl<'js> WithUserData for Ctx<'js> {
    fn user_data<'a>(&'a self) -> UserDataGuard<'a, JsUserData> {
        self.userdata::<JsUserData>().expect("userdata not set")
    }
}

#[derive(Constructor, Debug, JsLifetime)]
pub(crate) struct JsUserData {
    displays: Displays,
    screen: Screen,
    cancellation_token: CancellationToken,
    /// An optional scoped token (e.g. per-REPL-expression) whose children are
    /// cancelled independently of the root token. When set, `child_cancellation_token`
    /// returns a child of this token instead of the root one.
    scoped_cancellation_token: Mutex<Option<CancellationToken>>,
    rng: SharedRng,
    task_tracker: TaskTracker,
    script_engine: ScriptEngine,
    callbacks: Callbacks,
    platform: Platform,
}

impl JsUserData {
    pub(crate) fn displays(&self) -> Displays {
        self.displays.clone()
    }

    pub(crate) fn screen(&self) -> Screen {
        self.screen.clone()
    }

    pub(crate) fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub(crate) fn child_cancellation_token(&self) -> CancellationToken {
        let scoped = self.scoped_cancellation_token.lock();
        #[allow(clippy::option_if_let_else)]
        if let Some(token) = scoped.as_ref() {
            token.child_token()
        } else {
            self.cancellation_token.child_token()
        }
    }

    pub(crate) fn set_scoped_cancellation_token(&self, token: Option<CancellationToken>) {
        *self.scoped_cancellation_token.lock() = token;
    }

    pub(crate) fn rng(&self) -> SharedRng {
        self.rng.clone()
    }

    pub(crate) fn task_tracker(&self) -> TaskTracker {
        self.task_tracker.clone()
    }

    pub(crate) fn script_engine(&self) -> ScriptEngine {
        self.script_engine.clone()
    }

    pub(crate) const fn callbacks(&self) -> &Callbacks {
        &self.callbacks
    }

    pub(crate) const fn platform(&self) -> Platform {
        self.platform
    }

    pub(crate) fn require_linux<'js>(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        if self.platform.is_windows() {
            return Err(
                CommonError::UnsupportedPlatform("only available on Linux".into()).into_js(ctx),
            );
        }
        Ok(())
    }

    pub(crate) fn require_not_windows<'js>(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        if self.platform.is_windows() {
            return Err(
                CommonError::UnsupportedPlatform("not supported on Windows".into()).into_js(ctx),
            );
        }
        Ok(())
    }

    pub(crate) fn require_not_linux<'js>(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        if is_linux() {
            return Err(
                CommonError::UnsupportedPlatform("not supported on Linux".into()).into_js(ctx),
            );
        }
        Ok(())
    }
}

/// Should the script wait at the end of the execution?
/// @category App
/// @default `WaitAtEnd.Automatic`
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumIs,
    EnumIter,
    Eq,
    FromRepr,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[repr(u8)]
pub enum WaitAtEnd {
    /// Automatically decide if the script should wait.
    /// Setting hotstrings will have the script wait.
    #[default]
    /// `WaitAtEnd.Automatic`
    Automatic,

    /// Always wait.
    /// `WaitAtEnd.Yes`
    Yes,

    /// Never wait.
    /// `WaitAtEnd.No`
    No,
}

#[derive(Debug)]
pub struct RuntimeOptions {
    #[cfg(unix)]
    pub display_name: Option<String>,

    /// When false, the runtime will not install a Ctrl+C signal handler that
    /// cancels the root cancellation token. The caller is responsible for
    /// handling Ctrl+C (e.g. the REPL manages it per-expression).
    pub install_ctrl_c_handler: bool,

    /// Whether to create the system tray icon and menu.
    pub show_tray_icon: bool,

    /// Seed for the shared random number generator.
    /// When set, random-dependent APIs become deterministic.
    pub seed: Option<u64>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            #[cfg(unix)]
            display_name: None,
            install_ctrl_c_handler: true,
            show_tray_icon: true,
            seed: None,
        }
    }
}

#[derive_where(Debug)]
pub struct Runtime {
    #[cfg(unix)]
    runtime: Arc<x11::Runtime>,

    #[cfg(windows)]
    runtime: Arc<win::Runtime>,

    enigo: Arc<Mutex<Enigo>>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
    wait_at_end: AtomicU8,
    background_tasks_counter: AtomicU64,
    playing_sounds_tracker: Arc<PlayingSoundsTracker>,

    #[derive_where(skip)]
    clipboard: Clipboard,

    displays: Displays,
    platform: Platform,
    mouse: Mutex<Option<Mouse>>,
    keyboard: Mutex<Option<Keyboard>>,
    extensions: Extensions,
}

#[instrument(skip_all)]
fn new_enigo() -> Result<Arc<Mutex<Enigo>>> {
    Ok(Arc::new(Mutex::new(Enigo::new(&Settings::default())?)))
}

/// Disable OpenCV parallelism since we perform our own parallelism using rayon.
fn setup_opencv_threading() -> Result<()> {
    #[allow(clippy::redundant_closure_call)]
    (|| {
        opencv::opencv_branch_34! {
            {
                set_num_threads(0)
            } else {
                set_num_threads(1)
            }
        }
    })()?;

    Ok(())
}

#[cfg(all(test, unix))]
static TEST_X11_RUNTIME: OnceLock<Arc<x11::Runtime>> = OnceLock::new();

/// Shared clipboard for all tests.
///
/// `arboard::Clipboard` on Linux spawns a background OS thread to serve
/// clipboard selection requests. That thread is joined – and therefore *dies*
/// – when the last `arboard::Clipboard` handle is dropped. If the OS reuses
/// the dead thread's ID for the next test's clipboard thread, glibc's
/// per-thread tcache aliasing causes a double-free crash.
///
/// By keeping one `Clipboard` alive for the entire test-binary lifetime the
/// clipboard server thread never dies and the tcache is never corrupted.
#[cfg(test)]
static TEST_CLIPBOARD: OnceLock<Clipboard> = OnceLock::new();

#[cfg(unix)]
struct ActionaTray {
    cancellation_token: CancellationToken,
}

#[cfg(unix)]
impl ksni::Tray for ActionaTray {
    fn id(&self) -> String {
        "actiona-run".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        const ICON_PNG: &[u8] = include_bytes!("../../icons/icon.png");
        let Ok(img) = image::load_from_memory(ICON_PNG) else {
            return vec![];
        };
        let rgba = img.into_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba
            .pixels()
            .flat_map(|pixel| {
                let [red, green, blue, alpha] = pixel.0;
                [alpha, red, green, blue]
            })
            .collect();
        vec![ksni::Icon {
            width: i32::try_from(width).unwrap_or(0),
            height: i32::try_from(height).unwrap_or(0),
            data,
        }]
    }

    fn title(&self) -> String {
        "Actiona Run".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            ksni::menu::StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| {
                    this.cancellation_token.cancel();
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

#[cfg(windows)]
fn create_windows_tray_icon() -> Result<tray_icon::Icon> {
    const ICON_RESOURCE_ORDINAL: u16 = 1;
    const ICON_PNG: &[u8] = include_bytes!("../../icons/icon.png");

    match tray_icon::Icon::from_resource(ICON_RESOURCE_ORDINAL, None) {
        Ok(icon) => Ok(icon),
        Err(resource_error) => {
            warn!(
                "Failed to load embedded tray icon resource: {resource_error}. Falling back to PNG asset."
            );

            let image = image::load_from_memory(ICON_PNG)?;
            let rgba = image.into_rgba8();
            let (width, height) = rgba.dimensions();

            Ok(tray_icon::Icon::from_rgba(rgba.into_raw(), width, height)?)
        }
    }
}

#[cfg(windows)]
#[derive(Debug)]
enum UiEvent {
    Quit,
    MenuEvent(MenuEvent),
}

#[cfg(windows)]
struct App {
    tray_handle: Option<tray_icon::TrayIcon>,
    show_tray_icon: bool,
    is_shutting_down: Arc<AtomicBool>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
    proxy: EventLoopProxy<UiEvent>,
}

#[cfg(windows)]
impl ApplicationHandler<UiEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) && self.show_tray_icon {
            let menu = Menu::new();
            let quit_item = MenuItem::with_id("quit", "Quit", true, None);
            if let Err(err) = menu.append_items(&[&quit_item]) {
                error!("Failed to create tray menu: {err}");
            } else {
                match create_windows_tray_icon() {
                    Ok(icon) => match TrayIconBuilder::new()
                        .with_menu(Box::new(menu))
                        .with_tooltip("Actiona Run")
                        .with_icon(icon)
                        .build()
                    {
                        Ok(tray) => self.tray_handle = Some(tray),
                        Err(err) => error!("Failed to create tray icon: {err}"),
                    },
                    Err(err) => {
                        error!("Failed to load tray icon image: {err}");
                    }
                }
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UiEvent) {
        match event {
            UiEvent::Quit => {
                drop(self.tray_handle.take());
                event_loop.exit();
            }
            UiEvent::MenuEvent(menu_event) => {
                if menu_event.id == "quit" && !self.is_shutting_down.swap(true, Ordering::Relaxed) {
                    self.cancellation_token.cancel();
                    self.task_tracker.close();

                    let tracker = self.task_tracker.clone();
                    let quit_proxy = self.proxy.clone();
                    tokio::spawn(async move {
                        tracker.wait().await;
                        let _ = quit_proxy.send_event(UiEvent::Quit);
                    });
                }
            }
        }
    }
}

pub struct RuntimePlatformSetup {
    #[cfg(windows)]
    event_loop: Option<EventLoop<UiEvent>>,
}

impl RuntimePlatformSetup {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(show_tray_icon: bool) -> Result<Self> {
        #[cfg(not(windows))]
        let _ = show_tray_icon;
        Ok(Self {
            #[cfg(windows)]
            event_loop: if show_tray_icon {
                Some(EventLoop::<UiEvent>::with_user_event().build()?)
            } else {
                None
            },
        })
    }
}

impl Runtime {
    #[cfg(test)]
    fn test_tokio_runtime() -> &'static tokio::runtime::Runtime {
        static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

        TOKIO_RUNTIME.get_or_init(|| {
            // A single shared multi-thread runtime avoids per-test teardown
            // crashes (e.g. X11 connection exhaustion) and allows
            // block_in_place (which current_thread runtimes forbid).
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build shared Tokio test runtime")
        })
    }

    /// Initialises the single shared `x11::Runtime` used across all tests.
    ///
    /// Must be called from a **bare test thread** (not from within a Tokio
    /// `block_on` context) so that the inner `block_on` call is allowed.
    ///
    /// By reusing the same X11 connection and dedicated event-loop thread for
    /// the entire test binary we avoid two classes of failure:
    ///
    /// 1. **X11 "Maximum number of clients reached"** – each `X11Connection`
    ///    opens three sockets (async, sync, XCB). With hundreds of tests the
    ///    server-side limit of 256 clients is quickly exhausted if every test
    ///    opens its own connection.
    ///
    /// 2. **glibc tcache double-free / SIGSEGV** – `spawn_on_dedicated_thread`
    ///    creates an OS thread per `x11::Runtime`. When that thread dies glibc
    ///    doesn't fully flush its per-thread allocation cache (`tcache`).
    ///    If the OS reuses the thread ID for the next test's thread the two
    ///    tcache entries alias and a double-free is triggered.
    ///
    /// Both issues disappear when the thread (and its connections) live for the
    /// entire test-binary lifetime.
    #[cfg(all(test, unix))]
    fn test_x11_runtime_init() {
        if TEST_X11_RUNTIME.get().is_some() {
            return;
        }
        let cancellation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();
        let displays =
            crate::api::displays::Displays::new(cancellation_token.clone(), task_tracker.clone())
                .expect("failed to create Displays for shared test X11 runtime");
        let rt = Self::test_tokio_runtime()
            .block_on(x11::Runtime::new(
                cancellation_token,
                task_tracker,
                None,
                displays,
            ))
            .map(Arc::new)
            .expect("failed to create shared test X11 runtime");
        TEST_X11_RUNTIME.get_or_init(|| rt);
        Self::test_clipboard_init();
    }

    #[cfg(all(test, unix))]
    fn test_x11_runtime() -> Arc<x11::Runtime> {
        TEST_X11_RUNTIME
            .get()
            .cloned()
            .expect("test_x11_runtime_init() must be called before entering the async context")
    }

    /// Initialises the single shared `Clipboard` used across all tests.
    ///
    /// Must be called from a **bare test thread** before entering any
    /// `block_on` context.
    ///
    /// On Linux, `arboard::Clipboard::new()` spawns a background OS thread to
    /// serve clipboard-selection requests. That thread is joined – and therefore
    /// *dies* – when the last clipboard handle is dropped at the end of each test.
    /// glibc does not fully flush a dying thread's per-thread allocation cache
    /// (`tcache`). If the OS reuses the same thread ID for the next test's
    /// clipboard thread, the two tcache entries alias and a double-free results.
    ///
    /// Keeping one `Clipboard` alive for the entire test-binary lifetime
    /// prevents the clipboard thread from ever dying and eliminates the crash.
    #[cfg(test)]
    fn test_clipboard_init() {
        TEST_CLIPBOARD
            .get_or_init(|| Clipboard::new().expect("failed to create shared test clipboard"));
    }

    #[cfg(test)]
    fn test_clipboard() -> Clipboard {
        TEST_CLIPBOARD
            .get()
            .cloned()
            .expect("test_clipboard_init() must be called before entering the async context")
    }

    #[instrument(name = "Runtime::new", skip_all)]
    async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        #[cfg_attr(not(unix), allow(unused))] options: RuntimeOptions,
    ) -> Result<(Arc<Self>, ScriptEngine)> {
        let displays = Displays::new(cancellation_token.clone(), task_tracker.clone())?;

        #[cfg(unix)]
        let runtime = {
            #[cfg(test)]
            {
                Self::test_x11_runtime()
            }
            #[cfg(not(test))]
            {
                Arc::new(
                    x11::Runtime::new(
                        cancellation_token.clone(),
                        task_tracker.clone(),
                        options.display_name.as_deref(),
                        displays.clone(),
                    )
                    .await?,
                )
            }
        };

        #[cfg(windows)]
        let runtime = win::Runtime::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            displays.clone(),
        )
        .await?;

        setup_opencv_threading()?;

        task_tracker.spawn_blocking(|| {
            if let Err(err) = find_image::warm_up() {
                error!("Failed to warm up find_image: {}", err);
            }
        });

        let clipboard = {
            #[cfg(test)]
            {
                Self::test_clipboard()
            }
            #[cfg(not(test))]
            {
                Clipboard::new()?
            }
        };
        let platform = Platform::detect();
        let runtime = Arc::new(Self {
            runtime,
            enigo: new_enigo()?,
            cancellation_token: cancellation_token.clone(),
            task_tracker: task_tracker.clone(),
            #[allow(clippy::as_conversions)]
            wait_at_end: AtomicU8::new(WaitAtEnd::default() as u8),
            background_tasks_counter: AtomicU64::new(0),
            playing_sounds_tracker: Arc::new(PlayingSoundsTracker::default()),
            clipboard: clipboard.clone(),
            displays: displays.clone(),
            platform,
            mouse: Mutex::new(None),
            keyboard: Mutex::new(None),
            extensions: Extensions::new(task_tracker.clone(), cancellation_token.clone()).await?,
        });

        #[allow(clippy::option_if_let_else)]
        let rng = match options.seed {
            Some(seed) => SharedRng::from_seed(seed),
            None => SharedRng::default(),
        };

        let mouse_inner = Mouse::new(runtime.clone()).await?;
        let keyboard_inner = Keyboard::new(runtime.clone())?;
        *runtime.mouse.lock() = Some(mouse_inner.clone());
        *runtime.keyboard.lock() = Some(keyboard_inner.clone());
        let macro_player = Arc::new(
            MacroPlayer::new(runtime.clone(), keyboard_inner.clone(), mouse_inner.clone()).await?,
        );
        let app = JsApp::new(runtime.clone());
        let mouse =
            JsMouse::new(runtime.clone(), mouse_inner.clone(), macro_player.clone()).await?;
        let keyboard = JsKeyboard::new(
            runtime.clone(),
            keyboard_inner,
            macro_player.clone(),
            task_tracker.clone(),
            cancellation_token.clone(),
        )?;
        let console = JsConsole::default();
        let js_displays = JsDisplays::new(displays.clone())?;
        let windows_inner = Windows::new(runtime.clone());
        let screen_inner =
            Screen::new(runtime.clone(), displays.clone(), windows_inner.clone()).await?;
        let screen = JsScreen::new(screen_inner.clone());
        let clipboard = JsClipboard::new(clipboard);
        let system = JsSystem::new(task_tracker.clone()).await?;
        let audio = JsAudio::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            runtime.playing_sounds_tracker.clone(),
        )?;
        let process = JsProcess::new(task_tracker.clone());
        let notification = JsNotification::new(task_tracker.clone());
        let standard_paths = JsStandardPaths::default();
        let windows = JsWindows::new(windows_inner, screen_inner.clone());
        let macros = JsMacros::new(runtime.clone(), mouse_inner.clone(), macro_player).await?;

        let script_engine = ScriptEngine::new().await?;

        let local_rng = rng.clone();
        let local_script_engine = script_engine.clone();
        script_engine
            .with(|ctx| {
                let callbacks = Callbacks::new(
                    script_engine.context(),
                    cancellation_token.clone(),
                    task_tracker.clone(),
                );

                ctx.store_userdata(JsUserData::new(
                    displays,
                    screen_inner.clone(),
                    cancellation_token.clone(),
                    Mutex::new(None),
                    local_rng,
                    task_tracker.clone(),
                    local_script_engine,
                    callbacks,
                    platform,
                ))?;

                Self::register_classes(
                    ctx.clone(),
                    app,
                    mouse,
                    keyboard,
                    console,
                    js_displays,
                    screen,
                    clipboard,
                    task_tracker,
                    system,
                    audio,
                    process,
                    notification,
                    standard_paths,
                    windows,
                    macros,
                )?;

                Ok(())
            })
            .await?;

        Ok((runtime, script_engine))
    }

    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    fn register_classes(
        ctx: Ctx,
        app: JsApp,
        mouse: JsMouse,
        keyboard: JsKeyboard,
        console: JsConsole,
        js_displays: JsDisplays,
        screen: JsScreen,
        clipboard: JsClipboard,
        task_tracker: TaskTracker,
        system: JsSystem,
        audio: JsAudio,
        process: JsProcess,
        notification: JsNotification,
        standard_paths: JsStandardPaths,
        windows: JsWindows,
        macros: JsMacros,
    ) -> rquickjs::Result<()> {
        // Tools
        register_singleton_class::<JsConcurrency>(&ctx, JsConcurrency::new())?;
        global::register(&ctx)?;

        // Host classes
        register_host_class::<JsFile>(&ctx)?;
        register_host_class::<JsDirectory>(&ctx)?;
        register_host_class::<JsPath>(&ctx)?;
        register_host_class::<JsFilesystem>(&ctx)?;
        register_host_class::<JsAbortSignal>(&ctx)?;
        register_host_class::<JsMatch>(&ctx)?;

        // Value classes
        register_value_class::<JsPoint>(&ctx)?;
        register_value_class::<JsSize>(&ctx)?;
        register_value_class::<JsRect>(&ctx)?;
        register_value_class::<JsColor>(&ctx)?;
        register_value_class::<JsImage>(&ctx)?;
        register_value_class::<JsWildcard>(&ctx)?;
        register_value_class::<JsAbortController>(&ctx)?;

        // Singletons
        register_singleton_class::<JsApp>(&ctx, app)?;
        register_singleton_class::<JsMouse>(&ctx, mouse)?;
        register_singleton_class::<JsKeyboard>(&ctx, keyboard)?;
        register_singleton_class::<JsDialogs>(&ctx, JsDialogs::default())?;
        register_singleton_class::<JsConsole>(&ctx, console)?;
        register_singleton_class::<JsDisplays>(&ctx, js_displays)?;
        register_singleton_class::<JsScreen>(&ctx, screen)?;
        register_singleton_class::<JsClipboard>(&ctx, clipboard)?;
        register_singleton_class::<JsRandom>(&ctx, JsRandom::default())?;
        register_singleton_class::<JsDatetime>(&ctx, JsDatetime::default())?;
        register_singleton_class::<JsWeb>(&ctx, JsWeb::new(task_tracker))?;
        register_singleton_class::<JsSystem>(&ctx, system)?;
        register_singleton_class::<JsAudio>(&ctx, audio)?;
        register_singleton_class::<JsProcess>(&ctx, process)?;
        register_singleton_class::<JsNotification>(&ctx, notification)?;
        register_singleton_class::<JsStandardPaths>(&ctx, standard_paths)?;
        register_singleton_class::<JsWindows>(&ctx, windows)?;
        register_singleton_class::<JsMacros>(&ctx, macros)?;

        Ok(())
    }

    /// On Windows, when the OS re-launches this binary as a deep-link handler
    /// (e.g. Snipping Tool redirecting to `actiona-run://…`), writes the
    /// callback URL directly to the local socket that the originating
    /// `ask_screenshot` call is listening on, then returns `true` so the
    /// caller can exit immediately.
    ///
    /// Each `ask_screenshot` call owns its own uniquely-named socket, so
    /// routing is correct even when multiple actiona instances run in parallel.
    #[cfg(windows)]
    #[must_use]
    pub fn relay_deep_link_if_needed() -> bool {
        use std::io::Write as _;

        use interprocess::local_socket::{ConnectOptions, GenericNamespaced, ToNsName};

        let url_str = match std::env::args()
            .skip(1)
            .find(|arg| arg.starts_with("actiona-run://"))
        {
            Some(value) => value,
            None => return false,
        };

        // Extract the correlation ID to locate the right socket.
        let correlation_id = url::Url::parse(&url_str).ok().and_then(|u| {
            u.query_pairs()
                .find(|(key, _)| key == "x-request-correlation-id")
                .map(|(_, value)| value.into_owned())
        });

        let Some(correlation_id) = correlation_id else {
            return true; // our scheme but malformed — consume it
        };

        let socket_name = format!("actiona-screenclip-{correlation_id}");

        if let Ok(name) = socket_name.as_str().to_ns_name::<GenericNamespaced>()
            && let Ok(mut connection) = ConnectOptions::new().name(name).connect_sync()
        {
            let _ = connection.write_all(url_str.as_bytes());
        }

        true
    }

    #[instrument(skip_all)]
    pub async fn run<F, Fut>(
        platform: RuntimePlatformSetup,
        f: F,
        runtime_options: RuntimeOptions,
    ) -> Result<Vec<UnhandledException>>
    where
        F: FnOnce(Arc<Self>, ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        #[cfg(not(windows))]
        let _ = platform;

        let cancellation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();

        #[cfg(unix)]
        let unhandled_exceptions = {
            let show_tray_icon = runtime_options.show_tray_icon;

            let tray_handle = if show_tray_icon {
                let tray = ActionaTray {
                    cancellation_token: cancellation_token.clone(),
                };
                match tray.spawn().await {
                    Ok(handle) => Some(handle),
                    Err(err) => {
                        error!("Failed to create tray icon: {err}");
                        None
                    }
                }
            } else {
                None
            };

            let result = Self::run_impl(f, cancellation_token, task_tracker, runtime_options).await;

            if let Some(handle) = tray_handle {
                handle.shutdown().await;
            }

            result
        };

        #[cfg(windows)]
        let unhandled_exceptions = match platform.event_loop {
            Some(mut event_loop) => {
                use tokio::sync::oneshot;

                let (result_sender, result_receiver) = oneshot::channel();
                let is_shutting_down = Arc::new(AtomicBool::new(false));

                let proxy = event_loop.create_proxy();

                let menu_proxy = proxy.clone();
                MenuEvent::set_event_handler(Some(move |event| {
                    let _ = menu_proxy.send_event(UiEvent::MenuEvent(event));
                }));

                let local_cancellation_token = cancellation_token.clone();
                let local_task_tracker = task_tracker.clone();
                let task_is_shutting_down = is_shutting_down.clone();
                let task_proxy = proxy.clone();

                tokio::spawn(async move {
                    let unhandled_exceptions = Self::run_impl(
                        f,
                        local_cancellation_token,
                        local_task_tracker,
                        runtime_options,
                    )
                    .await;

                    if !task_is_shutting_down.swap(true, Ordering::Relaxed) {
                        let _ = task_proxy.send_event(UiEvent::Quit);
                    }

                    let _ = result_sender.send(unhandled_exceptions);
                });

                let mut app = App {
                    tray_handle: None,
                    show_tray_icon: true,
                    is_shutting_down,
                    cancellation_token,
                    task_tracker,
                    proxy,
                };

                event_loop.run_app_on_demand(&mut app)?;

                MenuEvent::set_event_handler(None::<fn(MenuEvent)>);

                result_receiver.await?
            }
            None => Self::run_impl(f, cancellation_token, task_tracker, runtime_options).await,
        };

        unhandled_exceptions
    }

    #[instrument(skip_all)]
    async fn run_impl<F, Fut>(
        f: F,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        runtime_options: RuntimeOptions,
    ) -> Result<Vec<UnhandledException>>
    where
        F: FnOnce(Arc<Self>, ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        if runtime_options.install_ctrl_c_handler {
            let local_cancellation_token = cancellation_token.clone();
            task_tracker.spawn(async move {
                select! {
                    _ = signal::ctrl_c() => {
                        local_cancellation_token.cancel();
                    },
                    _ = local_cancellation_token.cancelled() => {},
                }
            });
        }

        let (runtime, script_engine) = Self::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            runtime_options,
        )
        .await?;

        let js_runtime = script_engine.context().runtime().clone();
        // Use a dedicated token so the drive task keeps running until f() returns.
        // Cancelling the root token must not stop the drive task early, because
        // pending JS futures (e.g. sleep) need one more poll to see the
        // cancellation and settle their Promises; if the drive stops first,
        // into_future().await hangs indefinitely.
        let drive_cancellation_token = CancellationToken::new();
        let drive_token = drive_cancellation_token.clone();
        task_tracker.spawn(async move {
            _ = cancel_on(&drive_token, js_runtime.drive()).await;
        });

        f(runtime.clone(), script_engine.clone()).await?;

        let wait_at_end = runtime.wait_at_end();
        info!(
            "Wait at end: {}, background tasks: {}",
            wait_at_end,
            runtime.has_background_tasks()
        );
        if wait_at_end.is_yes() {
            cancellation_token.cancelled().await;
        } else if wait_at_end.is_automatic() && runtime.has_background_tasks() {
            while cancel_on(
                &cancellation_token,
                runtime.playing_sounds_tracker.notified(),
            )
            .await
            .is_ok()
            {
                if !runtime.has_background_tasks() {
                    break;
                }
            }
        }

        let unhandled_exceptions = script_engine.idle().await;

        // Remove userdata to break the reference cycle:
        // ScriptEngine -> AsyncContext -> JsUserData -> ScriptEngine
        script_engine
            .with(|ctx| {
                let _ = ctx.remove_userdata::<JsUserData>();
                Ok(())
            })
            .await
            .ok();
        // Cancel and wait for all background tasks (including the drive task)
        // BEFORE dropping script_engine or runtime.  The drive task holds the
        // InnerRuntime lock and executes QuickJS code; if the QuickJS context
        // is freed (via JS_FreeContext) while the drive task is still running,
        // the result is a use-after-free that manifests as the glibc
        // "double free detected in tcache 2" crash.
        //
        // Cancelling before dropping the runtime also ensures the X11 main
        // loop thread sees the cancellation and exits, so X11Runtime::drop's
        // thread.join() returns promptly.
        task_tracker.close();
        cancellation_token.cancel();
        drive_cancellation_token.cancel();

        task_tracker.wait().await;

        drop(script_engine);
        runtime.clear_runtime_back_references();
        drop(runtime);

        Result::Ok(unhandled_exceptions)
    }

    #[cfg(unix)]
    #[must_use]
    pub fn platform(&self) -> &x11::Runtime {
        &self.runtime
    }

    #[cfg(windows)]
    #[must_use]
    pub fn platform(&self) -> &win::Runtime {
        &self.runtime
    }

    #[must_use]
    pub fn mouse_buttons(&self) -> Guard<MouseButtonsTopic> {
        self.platform().mouse_buttons()
    }

    #[must_use]
    pub fn mouse_move(&self) -> Guard<MouseMoveTopic> {
        self.platform().mouse_move()
    }

    #[must_use]
    pub fn mouse_scroll(&self) -> Guard<MouseScrollTopic> {
        self.platform().mouse_scroll()
    }

    #[must_use]
    pub fn keyboard_keys(&self) -> Guard<KeyboardKeysTopic> {
        self.platform().keyboard_keys()
    }

    #[must_use]
    pub fn keyboard_text(&self) -> Guard<KeyboardTextTopic> {
        self.platform().keyboard_text()
    }

    #[must_use]
    pub fn mouse(&self) -> Option<Mouse> {
        self.mouse.lock().clone()
    }

    #[must_use]
    pub fn keyboard(&self) -> Option<Keyboard> {
        self.keyboard.lock().clone()
    }

    fn clear_runtime_back_references(&self) {
        self.mouse.lock().take();
        self.keyboard.lock().take();
    }

    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    #[must_use]
    pub fn task_tracker(&self) -> TaskTracker {
        self.task_tracker.clone()
    }

    #[must_use]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[must_use]
    pub fn enigo(&self) -> Arc<Mutex<Enigo>> {
        self.enigo.clone()
    }

    #[must_use]
    pub fn clipboard(&self) -> Clipboard {
        self.clipboard.clone()
    }

    #[must_use]
    pub fn displays(&self) -> Displays {
        self.displays.clone()
    }

    #[inline]
    pub fn block_on<F: Future<Output = R>, R>(f: F) -> R {
        block_in_place(|| -> R { Handle::current().block_on(f) })
    }

    #[cfg(test)]
    fn test_init() {
        // Initialise the shared X11 runtime while we are still on the bare
        // test thread, before entering block_on where a nested block_on
        // would panic.
        // test_x11_runtime_init() also calls test_clipboard_init() on unix.
        #[cfg(unix)]
        Self::test_x11_runtime_init();
        // On non-unix platforms the clipboard must be initialised separately.
        #[cfg(not(unix))]
        Self::test_clipboard_init();
    }

    #[cfg(test)]
    pub fn test<F, Fut>(f: F)
    where
        F: FnOnce(Arc<Self>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::test_init();

        let platform = RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new failed");

        Self::test_tokio_runtime()
            .block_on(async {
                Self::run(
                    platform,
                    async |runtime, script_engine| {
                        f(runtime).await;

                        let unhandled_exceptions = script_engine.idle().await;
                        assert!(
                            unhandled_exceptions.is_empty(),
                            "unhandled exceptions found: {unhandled_exceptions:?}"
                        );

                        Ok(())
                    },
                    RuntimeOptions {
                        show_tray_icon: false,
                        ..Default::default()
                    },
                )
                .await
            })
            .unwrap_or_else(|error| panic!("Runtime::test failed: {error:?}"));
    }

    #[cfg(test)]
    pub fn test_with_script_engine<F, Fut>(f: F)
    where
        F: FnOnce(ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::test_init();

        let platform = RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new failed");

        Self::test_tokio_runtime()
            .block_on(async {
                let unhandled_exceptions = Self::run(
                    platform,
                    async move |_runtime, script_engine| {
                        f(script_engine.clone()).await;

                        Ok(())
                    },
                    RuntimeOptions {
                        show_tray_icon: false,
                        ..Default::default()
                    },
                )
                .await?;

                assert!(
                    unhandled_exceptions.is_empty(),
                    "unhandled exceptions found: {unhandled_exceptions:?}"
                );

                Result::<()>::Ok(())
            })
            .unwrap_or_else(|error| panic!("Runtime::test_with_script_engine failed: {error:?}"));
    }

    pub fn set_wait_at_end(&self, wait_at_end: WaitAtEnd) {
        #[allow(clippy::as_conversions)]
        self.wait_at_end.store(wait_at_end as u8, Ordering::Relaxed);
    }

    pub fn wait_at_end(&self) -> WaitAtEnd {
        WaitAtEnd::from_repr(self.wait_at_end.load(Ordering::Relaxed))
            .unwrap_or(WaitAtEnd::Automatic)
    }

    pub fn increase_background_tasks_counter(&self) {
        self.background_tasks_counter
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrease_background_tasks_counter(&self) {
        if self
            .background_tasks_counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
                old.checked_sub(1)
            })
            .is_ok()
        {
            self.playing_sounds_tracker.notify_finished();
        } else {
            warn!("trying to decrement background_tasks_counter below 0");
        }
    }

    fn has_background_tasks(&self) -> bool {
        self.background_tasks_counter.load(Ordering::Relaxed) > 0
            || self.playing_sounds_tracker.has_playing_sounds()
    }

    pub fn require_not_wayland(&self) -> color_eyre::Result<()> {
        if self.platform.is_wayland() || self.platform.is_x_wayland() {
            return Err(CommonError::UnsupportedPlatform("not supported on Wayland".into()).into());
        }
        Ok(())
    }

    pub fn require_linux(&self) -> color_eyre::Result<()> {
        if self.platform.is_windows() {
            return Err(CommonError::UnsupportedPlatform("only available on Linux".into()).into());
        }
        Ok(())
    }

    pub fn require_not_windows(&self) -> color_eyre::Result<()> {
        if self.platform.is_windows() {
            return Err(CommonError::UnsupportedPlatform("not supported on Windows".into()).into());
        }
        Ok(())
    }

    pub fn require_not_linux(&self) -> color_eyre::Result<()> {
        if is_linux() {
            return Err(CommonError::UnsupportedPlatform("not supported on Linux".into()).into());
        }
        Ok(())
    }
}
