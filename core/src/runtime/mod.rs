use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
};

use color_eyre::Result;
use derive_more::Constructor;
use derive_where::derive_where;
use enigo::{Enigo, Settings};
use macros::{FromSerde, IntoSerde};
use opencv::core::set_num_threads;
use parking_lot::Mutex;
use rquickjs::{Ctx, JsLifetime, runtime::UserDataGuard};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumIter, FromRepr};
use tauri::{
    AppHandle,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tokio::{runtime::Handle, select, signal, sync::oneshot, task::block_in_place};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info, instrument, warn};

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
        directory::js::JsDirectory,
        displays::{Displays, js::JsDisplays},
        file::js::JsFile,
        filesystem::js::JsFilesystem,
        image::{find_image, js::JsImage},
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
        ui::js::JsUi,
        web::js::JsWeb,
        windows::{Windows, js::JsWindows},
    },
    cancel_on,
    error::CommonError,
    platform_info::Platform,
    runtime::{events::Guard, shared_rng::SharedRng},
    scripting::{Engine as ScriptEngine, UnhandledException, callbacks::Callbacks},
};

pub mod async_resource;
pub mod events;
pub mod platform;
pub mod shared_rng;

#[cfg(windows)]
use platform::win;
#[cfg(unix)]
use platform::x11;

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
    app_handle: Option<AppHandle>,
    script_engine: ScriptEngine,
    callbacks: Callbacks,
    no_globals: bool,
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

    pub(crate) fn app_handle(&self) -> AppHandle {
        self.app_handle
            .as_ref()
            .expect("Tauri app handle should be available")
            .clone()
    }

    pub(crate) fn script_engine(&self) -> ScriptEngine {
        self.script_engine.clone()
    }

    pub(crate) const fn callbacks(&self) -> &Callbacks {
        &self.callbacks
    }

    pub(crate) const fn no_globals(&self) -> bool {
        self.no_globals
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
        if self.platform.is_linux() {
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

    /// When true, all Actiona API objects are placed under an `actiona` namespace
    /// instead of the global scope.
    pub no_globals: bool,

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
            no_globals: false,
            install_ctrl_c_handler: true,
            show_tray_icon: true,
            seed: None,
        }
    }
}

#[derive_where(Debug)]
pub struct Runtime {
    #[cfg(unix)]
    runtime: x11::Runtime,

    #[cfg(windows)]
    runtime: Arc<win::Runtime>,

    enigo: Arc<Mutex<Enigo>>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
    app_handle: Option<AppHandle>,
    wait_at_end: AtomicU8,
    background_tasks_counter: AtomicU64,
    playing_sounds_tracker: Arc<PlayingSoundsTracker>,

    #[derive_where(skip)]
    clipboard: Clipboard,

    displays: Displays,
    platform: Platform,
    mouse: OnceLock<Mouse>,       // FIX: circular dependency
    keyboard: OnceLock<Keyboard>, // FIX: circular dependency
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

impl Runtime {
    #[cfg(test)]
    fn test_tokio_runtime() -> &'static tokio::runtime::Runtime {
        static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

        TOKIO_RUNTIME.get_or_init(|| {
            // Tests spin up native resources such as X11 and QuickJS. Sharing one
            // multithreaded runtime avoids per-test runtime teardown crashes while
            // still supporting block_in_place and background task progress.
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build shared Tokio test runtime")
        })
    }

    // TODO: make private
    #[instrument(name = "Runtime::new", skip_all)]
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        app_handle: Option<AppHandle>,
        #[cfg_attr(not(unix), allow(unused))] options: RuntimeOptions,
    ) -> Result<(Arc<Self>, ScriptEngine)> {
        let displays = Displays::new(cancellation_token.clone(), task_tracker.clone())?;

        #[cfg(unix)]
        let runtime = x11::Runtime::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            options.display_name.as_deref(),
            displays.clone(),
        )
        .await?;

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

        let clipboard = Clipboard::new()?;
        let platform = Platform::detect();
        let runtime = Arc::new(Self {
            runtime,
            enigo: new_enigo()?,
            cancellation_token: cancellation_token.clone(),
            task_tracker: task_tracker.clone(),
            app_handle: app_handle.clone(),
            #[allow(clippy::as_conversions)]
            wait_at_end: AtomicU8::new(WaitAtEnd::default() as u8),
            background_tasks_counter: AtomicU64::new(0),
            playing_sounds_tracker: Arc::new(PlayingSoundsTracker::default()),
            clipboard: clipboard.clone(),
            displays: displays.clone(),
            platform,
            mouse: OnceLock::new(),
            keyboard: OnceLock::new(),
        });

        #[allow(clippy::option_if_let_else)]
        let rng = match options.seed {
            Some(seed) => SharedRng::from_seed(seed),
            None => SharedRng::default(),
        };

        let mouse_inner = Mouse::new(runtime.clone()).await?;
        let keyboard_inner = Keyboard::new(runtime.clone())?;
        runtime.mouse.set(mouse_inner.clone()).ok();
        runtime.keyboard.set(keyboard_inner.clone()).ok();
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
                    app_handle,
                    local_script_engine,
                    callbacks,
                    options.no_globals,
                    platform,
                ))?;

                if options.no_globals {
                    let namespace = rquickjs::Object::new(ctx.clone())?;
                    ctx.globals().set("actiona", namespace)?;
                }

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
        JsConcurrency::register(&ctx)?;
        global::register(&ctx)?;

        // Host classes
        register_host_class::<JsFile>(&ctx)?;
        register_host_class::<JsDirectory>(&ctx)?;
        register_host_class::<JsPath>(&ctx)?;
        register_host_class::<JsFilesystem>(&ctx)?;
        register_host_class::<JsAbortSignal>(&ctx)?;

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
        register_singleton_class::<JsUi>(&ctx, JsUi::default())?;
        register_singleton_class::<JsConsole>(&ctx, console)?;
        register_singleton_class::<JsDisplays>(&ctx, js_displays)?;
        register_singleton_class::<JsScreen>(&ctx, screen)?;
        register_singleton_class::<JsClipboard>(&ctx, clipboard)?;
        register_singleton_class::<JsRandom>(&ctx, JsRandom::default())?;
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

    #[instrument(skip_all)]
    pub fn run_with_ui<F, Fut>(
        f: F,
        runtime_options: RuntimeOptions,
        tauri_context: tauri::Context<tauri_runtime_wry::Wry<tauri::EventLoopMessage>>,
    ) -> Result<Vec<UnhandledException>>
    where
        F: FnOnce(Arc<Self>, ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();
        let (result_sender, result_receiver) = oneshot::channel();

        let local_cancellation_token = cancellation_token.clone();
        let local_task_tracker = task_tracker.clone();
        let is_shutting_down = Arc::new(AtomicBool::new(false));
        let setup_is_shutting_down = is_shutting_down.clone();
        let show_tray_icon = runtime_options.show_tray_icon;
        let app = tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .setup(move |app| {
                let app_handle = app.handle().clone();
                let task_is_shutting_down = setup_is_shutting_down.clone();

                tauri::async_runtime::spawn(async move {
                    let unhandled_exceptions = Self::run_impl(
                        f,
                        local_cancellation_token,
                        local_task_tracker,
                        Some(app_handle.clone()),
                        runtime_options,
                    )
                    .await;

                    // If shutdown was already initiated (e.g. tray "Quit"), avoid
                    // issuing a second exit request after webviews are gone.
                    if !task_is_shutting_down.swap(true, Ordering::Relaxed) {
                        app_handle.exit(0);
                    }

                    let _ = result_sender.send(unhandled_exceptions);
                });

                if show_tray_icon {
                    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                    let show = MenuItem::with_id(app, "show", "Show Info", true, None::<&str>)?;
                    let menu = Menu::with_items(app, &[&show, &quit])?;

                    let mut tray_builder = TrayIconBuilder::new();
                    if let Some(icon) = app.default_window_icon() {
                        tray_builder = tray_builder.icon(icon.clone());
                    }

                    let _tray = tray_builder
                        .tooltip("Actiona 4 daemon") // hover text
                        .menu(&menu)
                        .show_menu_on_left_click(true) // default is true; set to false if you want left-click to do something else
                        .on_menu_event(|app, event| match event.id.as_ref() {
                            "quit" => app.exit(0),
                            "show" => {
                                println!("Tray -> Show Info clicked");
                                // do something: emit an event, open a window, etc.
                            }
                            _ => {}
                        })
                        .on_tray_icon_event(|_tray, event| {
                            if let TrayIconEvent::Click {
                                button: MouseButton::Left,
                                button_state: MouseButtonState::Up,
                                ..
                            } = event
                            {
                                // left click released; you could toggle a window here instead of showing the menu
                                println!("Left click on tray");
                            }
                        })
                        .build(app)?;
                }

                Ok(())
            })
            .build(tauri_context)?;

        app.run_return(move |app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if is_shutting_down.swap(true, Ordering::Relaxed) {
                    // We are already shutting down, don't prevent it.
                    return;
                }

                api.prevent_exit();
                cancellation_token.cancel();
                task_tracker.close();

                let app_handle = app_handle.clone();
                let tracker = task_tracker.clone();
                tauri::async_runtime::spawn(async move {
                    tracker.wait().await;
                    app_handle.exit(0);
                });
            }
        });

        let result = result_receiver.blocking_recv()??;

        Ok(result)
    }

    pub fn run<F, Fut>(f: F, runtime_options: RuntimeOptions) -> Result<Vec<UnhandledException>>
    where
        F: FnOnce(Arc<Self>, ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();

        #[cfg(test)]
        let unhandled_exceptions = Self::test_tokio_runtime().handle().block_on(async move {
            Self::run_impl(f, cancellation_token, task_tracker, None, runtime_options).await
        })?;

        #[cfg(not(test))]
        let unhandled_exceptions = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async move {
                Self::run_impl(f, cancellation_token, task_tracker, None, runtime_options).await
            })?;

        Ok(unhandled_exceptions)
    }

    #[instrument(skip_all)]
    async fn run_impl<F, Fut>(
        f: F,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        app_handle: Option<AppHandle>,
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
            app_handle,
            runtime_options,
        )
        .await?;

        let js_runtime = script_engine.context().runtime().clone();
        let drive_token = cancellation_token.clone();
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
        drop(script_engine);
        drop(runtime);

        task_tracker.close();
        cancellation_token.cancel();

        task_tracker.wait().await;

        Result::Ok(unhandled_exceptions)
    }

    #[cfg(unix)]
    #[must_use]
    pub const fn platform(&self) -> &x11::Runtime {
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
        self.mouse.get().cloned()
    }

    #[must_use]
    pub fn keyboard(&self) -> Option<Keyboard> {
        self.keyboard.get().cloned()
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
    pub fn enigo(&self) -> Arc<Mutex<Enigo>> {
        self.enigo.clone()
    }

    #[must_use]
    pub const fn tauri_app(&self) -> &AppHandle {
        self.app_handle
            .as_ref()
            .expect("tauri_app() requires Runtime to be initialized with a Tauri app handle")
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

    pub fn test<F, Fut>(f: F)
    where
        F: FnOnce(Arc<Self>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::run(
            async |runtime, script_engine| {
                f(runtime).await;

                let unhandled_exceptions = script_engine.idle().await;
                assert!(
                    unhandled_exceptions.is_empty(),
                    "unhandled exceptions found: {unhandled_exceptions:?}"
                );

                Ok(())
            },
            RuntimeOptions::default(),
        )
        .unwrap_or_else(|error| panic!("Runtime::test failed: {error:?}"));
    }

    pub fn test_with_ui<F, Fut>(f: F)
    where
        F: FnOnce(Arc<Self>, ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let unhandled_exceptions = Self::run_with_ui(
            async |runtime, script_engine| {
                f(runtime, script_engine.clone()).await;

                Ok(())
            },
            RuntimeOptions::default(),
            tauri::generate_context!(),
        )
        .unwrap_or_else(|error| panic!("Runtime::test_with_ui failed: {error:?}"));

        assert!(
            unhandled_exceptions.is_empty(),
            "unhandled exceptions found: {unhandled_exceptions:?}"
        );
    }

    pub fn test_with_script_engine<F, Fut>(f: F)
    where
        F: FnOnce(ScriptEngine) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let unhandled_exceptions = Self::run(
            async move |_runtime, script_engine| {
                f(script_engine.clone()).await;

                Ok(())
            },
            RuntimeOptions::default(),
        )
        .unwrap_or_else(|error| panic!("Runtime::test_with_script_engine failed: {error:?}"));

        assert!(
            unhandled_exceptions.is_empty(),
            "unhandled exceptions found: {unhandled_exceptions:?}"
        );
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
        if self.platform.is_wayland() {
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
        if self.platform.is_linux() {
            return Err(CommonError::UnsupportedPlatform("not supported on Linux".into()).into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Display;
    use macros::{FromSerde, IntoSerde, js_class, js_methods};
    use rquickjs::{Function, Object, Value, atom::PredefinedAtom, class::Trace};
    use serde::{Deserialize, Serialize};
    use strum::EnumIter;

    use super::*;
    use crate::api::js::classes::{
        SingletonClass, ValueClass, register_enum, register_singleton_class,
    };

    fn print<'js>(value: Value<'js>) {
        println!("{value:?}");
    }

    #[derive(
        Clone,
        Copy,
        Debug,
        Deserialize,
        Display,
        EnumIter,
        Eq,
        FromSerde,
        IntoSerde,
        PartialEq,
        Serialize,
    )]
    enum TestEnum {
        A,
        B,
    }

    #[derive(Clone, JsLifetime, Trace)]
    #[js_class]
    pub struct JsTestGenerator {
        n: i32,
    }

    #[js_methods]
    impl JsTestGenerator {
        #[qjs(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self { n: 0 }
        }

        #[qjs(rename = PredefinedAtom::SymbolAsyncIterator)]
        pub fn async_iter(&self) -> Self {
            self.clone()
        }

        pub async fn next<'js>(&mut self, ctx: Ctx<'js>) -> rquickjs::Result<Value<'js>> {
            let obj = Object::new(ctx)?;
            obj.prop("done", self.n == 2)?;
            obj.prop("value", self.n)?;
            self.n += 1;

            Ok(obj.into_value())
        }
    }

    impl ValueClass<'_> for JsTestGenerator {}

    #[derive(Default, JsLifetime, Trace)]
    #[js_class]
    pub struct JsTestSingletonStruct {
        string: String,
        integer: i64,
        float: f64,
    }

    #[js_methods]
    impl JsTestSingletonStruct {}

    impl SingletonClass<'_> for JsTestSingletonStruct {}

    async fn setup(script_engine: ScriptEngine) {
        script_engine
            .with(|ctx| {
                ctx.globals()
                    .prop("print", Function::new(ctx.clone(), print))?;
                register_enum::<TestEnum>(&ctx)?;
                register_singleton_class::<JsTestSingletonStruct>(
                    &ctx,
                    JsTestSingletonStruct::default(),
                )?;
                register_value_class::<JsTestGenerator>(&ctx)?;
                Ok(())
            })
            .await
            .unwrap();
    }

    #[test]
    #[ignore]
    fn test_singleton() {
        Runtime::test_with_script_engine(|script_engine| async move {
            setup(script_engine.clone()).await;

            script_engine
                .eval_async::<()>(
                    r#"
                const gen = TestGenerator();
                for await (const ev of gen) {
                    print(ev);
                } // TODO
                /*
                test_singleton_struct.string = "foo";
                test_singleton_struct.integer = 42;
                test_singleton_struct.float = 42.5;
                test_singleton_struct.setCallback(() => { print("callback called"); });
                test_singleton_struct.call();
                test_singleton_struct.clear();
                */
                //test_singleton_struct.setRustCallback(() => { print("rust callback called"); });
                //test_singleton_struct.callRust();
                "#,
                )
                .await
                .unwrap();
        });
    }
}
