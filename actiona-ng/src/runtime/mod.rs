use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use enigo::{Enigo, Settings};
use eyre::{Result, eyre};
use rquickjs::{Ctx, JsLifetime, runtime::UserDataGuard};
use tauri::AppHandle;
use tokio::{runtime::Handle, select, signal, task::block_in_place};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    core::{
        clipboard::js::JsClipboard,
        color::js::JsColor,
        console::js::JsConsole,
        directory::js::JsDirectory,
        displays::{Displays, js::JsDisplays},
        file::js::JsFile,
        filesystem::js::JsFilesystem,
        image::js::JsImage,
        js::{
            abort_controller::{JsAbortController, JsAbortSignal},
            classes::{register_singleton_class, register_value_class},
            concurrency::JsConcurrency,
            global,
        },
        keyboard::js::JsKeyboard,
        mouse::js::JsMouse,
        name::js::{JsName, JsWildcard},
        path::js::JsPath,
        point::js::JsPoint,
        random::js::JsRandom,
        rect::js::JsRect,
        screenshot::js::JsScreenshot,
        size::js::JsSize,
        ui::js::JsUi,
        web::js::JsWeb,
    },
    runtime::shared_rng::SharedRng,
    scripting::Engine as ScriptEngine,
};

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

#[derive(Debug, JsLifetime)]
pub(crate) struct JsUserData {
    displays: Arc<Displays>,
    cancellation_token: CancellationToken,
    rng: SharedRng,
    task_tracker: TaskTracker,
    app_handle: Option<AppHandle>,
}

impl JsUserData {
    const fn new(
        displays: Arc<Displays>,
        cancellation_token: CancellationToken,
        rng: SharedRng,
        task_tracker: TaskTracker,
        app_handle: Option<AppHandle>,
    ) -> Self {
        Self {
            displays,
            cancellation_token,
            rng,
            task_tracker,
            app_handle,
        }
    }

    pub(crate) fn displays(&self) -> Arc<Displays> {
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
}

#[derive(Debug)]
pub struct Runtime {
    #[cfg(unix)]
    runtime: x11::Runtime,

    #[cfg(windows)]
    runtime: Arc<win::Runtime>,

    enigo: Arc<Mutex<Enigo>>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
    app_handle: Option<AppHandle>,
}

impl Runtime {
    // TODO: make private
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        app_handle: Option<AppHandle>,
    ) -> Result<(Arc<Self>, Arc<ScriptEngine>)> {
        #[cfg(unix)]
        let runtime = x11::Runtime::new(cancellation_token.clone(), task_tracker.clone()).await?;

        #[cfg(windows)]
        let runtime = win::Runtime::new(cancellation_token.clone(), task_tracker.clone()).await?;

        let runtime = Arc::new(Self {
            runtime,
            enigo: Arc::new(Mutex::new(Enigo::new(&Settings::default())?)),
            cancellation_token: cancellation_token.clone(),
            task_tracker: task_tracker.clone(),
            app_handle: app_handle.clone(),
        });

        let displays = Arc::new(Displays::new(runtime.clone())?);

        let rng = SharedRng::default();

        let mouse = JsMouse::new(runtime.clone()).await?;
        let keyboard = JsKeyboard::new(runtime.clone())?;
        let console = JsConsole::new(runtime.clone())?;
        let js_displays = JsDisplays::new(displays.clone())?;
        let screenshot = JsScreenshot::new(runtime.clone(), displays.clone()).await?;

        let script_engine = ScriptEngine::new().await?;

        let local_rng = rng.clone();
        script_engine
            .with(|ctx| -> Result<()> {
                ctx.store_userdata(JsUserData::new(
                    displays,
                    cancellation_token.clone(),
                    local_rng,
                    task_tracker.clone(),
                    app_handle,
                ))
                .unwrap();

                (|| -> rquickjs::Result<()> {
                    // Tools
                    JsConcurrency::register(&ctx)?;
                    global::register(&ctx)?;

                    // Value classes
                    register_value_class::<JsPoint>(&ctx)?;
                    register_value_class::<JsSize>(&ctx)?;
                    register_value_class::<JsRect>(&ctx)?;
                    register_value_class::<JsColor>(&ctx)?;
                    register_value_class::<JsImage>(&ctx)?;
                    register_value_class::<JsFile>(&ctx)?;
                    register_value_class::<JsWildcard>(&ctx)?;
                    register_value_class::<JsName>(&ctx)?;
                    register_value_class::<JsDirectory>(&ctx)?;
                    register_value_class::<JsPath>(&ctx)?;
                    register_value_class::<JsFilesystem>(&ctx)?;
                    register_value_class::<JsAbortSignal>(&ctx)?;
                    register_value_class::<JsAbortController>(&ctx)?;

                    // Singletons
                    register_singleton_class::<JsMouse>(&ctx, mouse)?;
                    register_singleton_class::<JsKeyboard>(&ctx, keyboard)?;
                    register_singleton_class::<JsUi>(&ctx, JsUi::default())?;
                    register_singleton_class::<JsConsole>(&ctx, console)?;
                    register_singleton_class::<JsDisplays>(&ctx, js_displays)?;
                    register_singleton_class::<JsScreenshot>(&ctx, screenshot)?;
                    register_singleton_class::<JsClipboard>(&ctx, JsClipboard::new(&ctx)?)?;
                    register_singleton_class::<JsRandom>(&ctx, JsRandom::default())?;
                    register_singleton_class::<JsWeb>(&ctx, JsWeb::new(task_tracker))?;

                    Ok(())
                })()
                .map_err(|_| {
                    let caught_error = ctx.catch();
                    eyre!("registration error: {:?}", caught_error)
                })?;

                Ok(())
            })
            .await?;

        Ok((runtime, Arc::new(script_engine)))
    }

    pub fn run_with_ui<F, Fut>(
        f: F,
        tauri_context: tauri::Context<tauri_runtime_wry::Wry<tauri::EventLoopMessage>>,
    ) -> Result<()>
    where
        F: FnOnce(Arc<Self>, Arc<ScriptEngine>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();

        let local_cancellation_token = cancellation_token.clone();
        let local_task_tracker = task_tracker.clone();
        let app = tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .setup(move |app| {
                let app_handle = app.handle().clone();

                tauri::async_runtime::spawn(async move {
                    Self::run_impl(
                        f,
                        local_cancellation_token,
                        local_task_tracker,
                        Some(app_handle.clone()),
                    )
                    .await
                    .unwrap();

                    app_handle.exit(0);
                });

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

        Ok(())
    }

    pub fn run<F, Fut>(f: F) -> Result<()>
    where
        F: FnOnce(Arc<Self>, Arc<ScriptEngine>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let task_tracker = TaskTracker::new();

        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        tokio_runtime.block_on(async move {
            Self::run_impl(f, cancellation_token, task_tracker, None).await
        })?;

        Ok(())
    }

    async fn run_impl<F, Fut>(
        f: F,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        app_handle: Option<AppHandle>,
    ) -> Result<()>
    where
        F: FnOnce(Arc<Self>, Arc<ScriptEngine>) -> Fut + Send + 'static,
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

        let (runtime, script_engine) =
            Self::new(cancellation_token.clone(), task_tracker.clone(), app_handle).await?;

        f(runtime, script_engine.clone()).await?;

        // TODO: proper error
        let unhandled_exceptions = script_engine.idle().await;
        assert!(
            unhandled_exceptions.is_empty(),
            "unhandled exceptions found: {unhandled_exceptions:?}"
        );

        task_tracker.close();
        cancellation_token.cancel();

        task_tracker.wait().await;

        Result::<()>::Ok(())
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

    #[inline]
    pub fn block_on<F: Future<Output = R>, R>(f: F) -> R {
        block_in_place(|| -> R { Handle::current().block_on(f) })
    }

    pub fn test<F, Fut>(f: F)
    where
        F: FnOnce(Arc<Self>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::run(async |runtime, script_engine| {
            f(runtime).await;

            let unhandled_exceptions = script_engine.idle().await;
            assert!(
                unhandled_exceptions.is_empty(),
                "unhandled exceptions found: {unhandled_exceptions:?}"
            );

            Ok(())
        })
        .unwrap();
    }

    pub fn test_with_ui<F, Fut>(f: F)
    where
        F: FnOnce(Arc<Self>, Arc<ScriptEngine>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::run_with_ui(
            async |runtime, script_engine| {
                f(runtime, script_engine.clone()).await;

                let unhandled_exceptions = script_engine.idle().await;
                assert!(
                    unhandled_exceptions.is_empty(),
                    "unhandled exceptions found: {unhandled_exceptions:?}"
                );

                Ok(())
            },
            tauri::generate_context!(),
        )
        .unwrap();
    }

    pub fn test_with_script_engine<F, Fut>(f: F)
    where
        F: FnOnce(Arc<ScriptEngine>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::run(async move |_runtime, script_engine| {
            f(script_engine.clone()).await;

            let unhandled_exceptions = script_engine.idle().await;
            assert!(
                unhandled_exceptions.is_empty(),
                "unhandled exceptions found: {unhandled_exceptions:?}"
            );

            Ok(())
        })
        .unwrap();
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
    use crate::core::js::classes::{
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

    async fn setup(script_engine: Arc<ScriptEngine>) {
        script_engine
            .with(|ctx| {
                ctx.globals()
                    .prop("print", Function::new(ctx.clone(), print))
                    .unwrap();
                register_enum::<TestEnum>(&ctx).unwrap();
                register_singleton_class::<TestSingletonStruct>(
                    &ctx,
                    TestSingletonStruct::default(),
                )
                .unwrap();
                register_value_class::<TestGenerator>(&ctx).unwrap();
            })
            .await;
    }

    #[test]
    fn test_enum() {
        Runtime::test_with_script_engine(|script_engine| async move {
            setup(script_engine.clone()).await;

            let result = script_engine.eval::<TestEnum>("TestEnum.B").await.unwrap();
            assert_eq!(result, TestEnum::B);
        });
    }

    #[test]
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
