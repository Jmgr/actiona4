use std::sync::{
    Arc,
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
    KeyboardKeysTopic, KeyboardTextTopic, MouseButtonsTopic, MouseMoveTopic,
};
#[cfg(windows)]
use crate::runtime::win::events::input::{
    keyboard::KeyboardKeysTopic, keyboard::KeyboardTextTopic, mouse::MouseButtonsTopic,
    mouse::MouseMoveTopic,
};
use crate::{
    api::{
        app::js::JsApp,
        audio::js::JsAudio,
        clipboard::{Clipboard, js::JsClipboard},
        color::js::JsColor,
        console::js::JsConsole,
        directory::js::JsDirectory,
        displays::{Displays, js::JsDisplays},
        file::js::JsFile,
        filesystem::js::JsFilesystem,
        hotstrings::js::JsHotstrings,
        image::{find_image, js::JsImage},
        js::{
            abort_controller::{JsAbortController, JsAbortSignal},
            classes::{register_host_class, register_singleton_class, register_value_class},
            concurrency::JsConcurrency,
            global,
        },
        keyboard::js::JsKeyboard,
        mouse::js::JsMouse,
        name::js::JsWildcard,
        path::js::JsPath,
        point::js::JsPoint,
        process::js::JsProcess,
        random::js::JsRandom,
        rect::js::JsRect,
        screenshot::js::JsScreenshot,
        size::js::JsSize,
        standardpaths::js::JsStandardPaths,
        system::js::JsSystem,
        ui::js::JsUi,
        web::js::JsWeb,
        windows::js::JsWindows,
    },
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
    cancellation_token: CancellationToken,
    rng: SharedRng,
    task_tracker: TaskTracker,
    app_handle: Option<AppHandle>,
    script_engine: ScriptEngine,
    callbacks: Callbacks,
    no_globals: bool,
}

impl JsUserData {
    pub(crate) fn displays(&self) -> Displays {
        self.displays.clone()
    }

    pub(crate) fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub(crate) fn child_cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.child_token()
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
}

/// Should the script wait at the end of the execution?
/// @category App
/// @default `WaitAtEnd.Automatic`
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
    Automatic,

    /// Always wait.
    Yes,

    /// Never wait.
    No,
}

#[derive(Debug, Default)]
pub struct RuntimeOptions {
    #[cfg(unix)]
    pub display_name: Option<String>,
    /// When true, all Actiona API objects are placed under an `actiona` namespace
    /// instead of the global scope.
    pub no_globals: bool,
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

    #[derive_where(skip)]
    clipboard: Clipboard,
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
        let runtime = Arc::new(Self {
            runtime,
            enigo: new_enigo()?,
            cancellation_token: cancellation_token.clone(),
            task_tracker: task_tracker.clone(),
            app_handle: app_handle.clone(),
            #[allow(clippy::as_conversions)]
            wait_at_end: AtomicU8::new(WaitAtEnd::default() as u8),
            background_tasks_counter: AtomicU64::new(0),
            clipboard: clipboard.clone(),
        });

        let rng = SharedRng::default();

        let app = JsApp::new(runtime.clone());
        let mouse = JsMouse::new(runtime.clone()).await?;
        let keyboard = JsKeyboard::new(runtime.clone())?;
        let console = JsConsole::default();
        let js_displays = JsDisplays::new(displays.clone())?;
        let screenshot = JsScreenshot::new(runtime.clone(), displays.clone()).await?;
        let clipboard = JsClipboard::new(clipboard);
        let system = JsSystem::new(task_tracker.clone()).await?;
        let hotstrings = JsHotstrings::new(
            runtime.clone(),
            task_tracker.clone(),
            cancellation_token.clone(),
        );
        let audio = JsAudio::new(cancellation_token.clone(), task_tracker.clone())?;
        let process = JsProcess::new(task_tracker.clone());
        let standard_paths = JsStandardPaths::default();
        let windows = JsWindows::new(runtime.clone());

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
                    cancellation_token.clone(),
                    local_rng,
                    task_tracker.clone(),
                    app_handle,
                    local_script_engine,
                    callbacks,
                    options.no_globals,
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
                    screenshot,
                    clipboard,
                    task_tracker,
                    system,
                    hotstrings,
                    audio,
                    process,
                    standard_paths,
                    windows,
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
        screenshot: JsScreenshot,
        clipboard: JsClipboard,
        task_tracker: TaskTracker,
        system: JsSystem,
        hotstrings: JsHotstrings,
        audio: JsAudio,
        process: JsProcess,
        standard_paths: JsStandardPaths,
        windows: JsWindows,
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
        register_singleton_class::<JsScreenshot>(&ctx, screenshot)?;
        register_singleton_class::<JsClipboard>(&ctx, clipboard)?;
        register_singleton_class::<JsRandom>(&ctx, JsRandom::default())?;
        register_singleton_class::<JsWeb>(&ctx, JsWeb::new(task_tracker))?;
        register_singleton_class::<JsSystem>(&ctx, system)?;
        register_singleton_class::<JsHotstrings>(&ctx, hotstrings)?;
        register_singleton_class::<JsAudio>(&ctx, audio)?;
        register_singleton_class::<JsProcess>(&ctx, process)?;
        register_singleton_class::<JsStandardPaths>(&ctx, standard_paths)?;
        register_singleton_class::<JsWindows>(&ctx, windows)?;

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
        let app = tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .setup(move |app| {
                let app_handle = app.handle().clone();

                tauri::async_runtime::spawn(async move {
                    let unhandled_exceptions = Self::run_impl(
                        f,
                        local_cancellation_token,
                        local_task_tracker,
                        Some(app_handle.clone()),
                        runtime_options,
                    )
                    .await;

                    app_handle.exit(0);

                    let _ = result_sender.send(unhandled_exceptions);
                });

                let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let show = MenuItem::with_id(app, "show", "Show Info", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show, &quit])?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone()) // or .title("Actiona 4")
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

                Ok(())
            })
            .build(tauri_context)?;

        let is_shutting_down = AtomicBool::new(false);

        app.run_return(move |app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if is_shutting_down.swap(true, Ordering::AcqRel) {
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

        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let unhandled_exceptions = tokio_runtime.block_on(async move {
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
        let local_cancellation_token = cancellation_token.clone();
        task_tracker.spawn(async move {
            select! {
                _ = signal::ctrl_c() => {
                    local_cancellation_token.cancel();
                },
                _ = local_cancellation_token.cancelled() => {},
            }
        });

        let (runtime, script_engine) = Self::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            app_handle,
            runtime_options,
        )
        .await?;

        f(runtime.clone(), script_engine.clone()).await?;

        // TODO: wait if there is a sound playing
        let wait_at_end = runtime.wait_at_end();
        info!(
            "Wait at end: {}, background tasks: {}",
            wait_at_end,
            runtime.has_background_tasks()
        );
        if wait_at_end.is_yes() || (wait_at_end.is_automatic() && runtime.has_background_tasks()) {
            cancellation_token.cancelled().await;
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
    pub fn keyboard_keys(&self) -> Guard<KeyboardKeysTopic> {
        self.platform().keyboard_keys()
    }

    #[must_use]
    pub fn keyboard_text(&self) -> Guard<KeyboardTextTopic> {
        self.platform().keyboard_text()
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
        self.app_handle.as_ref().unwrap()
    }

    #[must_use]
    pub fn clipboard(&self) -> Clipboard {
        self.clipboard.clone()
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
        .unwrap();
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
        .unwrap();

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
        .unwrap();

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
        WaitAtEnd::from_repr(self.wait_at_end.load(Ordering::Relaxed)).unwrap()
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
            .is_err()
        {
            warn!("trying to decrement background_tasks_counter below 0");
        }
    }

    fn has_background_tasks(&self) -> bool {
        self.background_tasks_counter.load(Ordering::Relaxed) > 0
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Display;
    use macros::{FromSerde, IntoSerde};
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
    #[rquickjs::class]
    pub struct TestGenerator {
        n: i32,
    }

    #[rquickjs::methods(rename_all = "camelCase")]
    impl TestGenerator {
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

    impl ValueClass<'_> for TestGenerator {}

    #[derive(Default, JsLifetime, Trace)]
    #[rquickjs::class]
    pub struct TestSingletonStruct {
        string: String,
        integer: i64,
        float: f64,
    }

    #[rquickjs::methods(rename_all = "camelCase")]
    impl TestSingletonStruct {}

    impl SingletonClass<'_> for TestSingletonStruct {}

    async fn setup(script_engine: ScriptEngine) {
        script_engine
            .with(|ctx| {
                ctx.globals()
                    .prop("print", Function::new(ctx.clone(), print))?;
                register_enum::<TestEnum>(&ctx)?;
                register_singleton_class::<TestSingletonStruct>(
                    &ctx,
                    TestSingletonStruct::default(),
                )?;
                register_value_class::<TestGenerator>(&ctx)?;
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
                }
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
