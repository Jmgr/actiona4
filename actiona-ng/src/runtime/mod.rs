use std::sync::{Arc, Mutex};

use async_compat::Compat;
use enigo::{Enigo, Settings};
use eyre::{Result, eyre};
use rquickjs::{Ctx, JsLifetime, runtime::UserDataGuard};
use tokio::{runtime::Handle, select, signal, task::block_in_place};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[cfg(feature = "slint")]
use crate::core::ui::js::JsUi;
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
            classes::{SingletonClass, ValueClass},
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
}

impl JsUserData {
    fn new(
        displays: Arc<Displays>,
        cancellation_token: CancellationToken,
        rng: SharedRng,
        task_tracker: TaskTracker,
    ) -> Self {
        Self {
            displays,
            cancellation_token,
            rng,
            task_tracker,
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
}

impl Runtime {
    // TODO: make private
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<(Arc<Self>, ScriptEngine)> {
        #[cfg(unix)]
        let runtime = x11::Runtime::new(cancellation_token.clone(), task_tracker.clone()).await?;

        #[cfg(windows)]
        let runtime = win::Runtime::new(cancellation_token.clone(), task_tracker.clone()).await?;

        let runtime = Arc::new(Self {
            runtime,
            enigo: Arc::new(Mutex::new(Enigo::new(&Settings::default())?)),
            cancellation_token: cancellation_token.clone(),
            task_tracker: task_tracker.clone(),
        });

        let displays = Arc::new(Displays::new(runtime.clone())?);

        let rng = SharedRng::default();

        let mouse = JsMouse::new(runtime.clone()).await?;
        let keyboard = JsKeyboard::new(runtime.clone())?;
        #[cfg(feature = "slint")]
        let ui = JsUi::new(runtime.clone(), displays.clone())?;
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
                ))
                .unwrap();

                (|| -> rquickjs::Result<()> {
                    // Tools
                    JsConcurrency::register(&ctx)?;
                    global::register(&ctx)?;

                    // Value classes
                    JsPoint::register(&ctx)?;
                    JsSize::register(&ctx)?;
                    JsRect::register(&ctx)?;
                    JsColor::register(&ctx)?;
                    JsImage::register(&ctx)?;
                    JsFile::register(&ctx)?;
                    JsWildcard::register(&ctx)?;
                    JsName::register(&ctx)?;
                    JsDirectory::register(&ctx)?;
                    JsPath::register(&ctx)?;
                    JsFilesystem::register(&ctx)?;
                    JsAbortSignal::register(&ctx)?;
                    JsAbortController::register(&ctx)?;

                    // Singletons
                    JsMouse::register(&ctx, mouse)?;
                    JsKeyboard::register(&ctx, keyboard)?;
                    #[cfg(feature = "slint")]
                    JsUi::register(&ctx, ui)?;
                    JsConsole::register(&ctx, console)?;
                    JsDisplays::register(&ctx, js_displays)?;
                    JsScreenshot::register(&ctx, screenshot)?;
                    JsClipboard::register(&ctx, JsClipboard::new(&ctx)?)?;
                    JsRandom::register(&ctx, JsRandom::default())?;
                    JsWeb::register(&ctx, JsWeb::new(task_tracker))?;

                    Ok(())
                })()
                .map_err(|_| {
                    let caught_error = ctx.catch();
                    eyre!("registration error: {:?}", caught_error)
                })?;

                Ok(())
            })
            .await?;

        Ok((runtime, script_engine))
    }

    #[cfg(feature = "slint")]
    pub fn run_with_ui<F>(f: F) -> Result<()>
    where
        F: AsyncFnOnce(Arc<Self>, &mut ScriptEngine) -> Result<()> + 'static,
    {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = tokio_runtime.enter();

        let cancellation_token = CancellationToken::new();
        let local_cancellation_token = cancellation_token.clone();

        let task_tracker = TaskTracker::new();
        let local_task_tracker = task_tracker.clone();

        task_tracker.spawn(async move {
            select! {
                _ = signal::ctrl_c() => {
                    slint::quit_event_loop().unwrap();
                    local_cancellation_token.cancel();
                },
                _ = local_cancellation_token.cancelled() => {},
            }
        });

        let local_cancellation_token = cancellation_token.clone();

        let handle = slint::spawn_local(Compat::new(async move {
            let (runtime, mut script_engine) =
                Self::new(local_cancellation_token, local_task_tracker).await?;

            f(runtime, &mut script_engine).await?;

            // TODO: proper error
            let unhandled_exceptions = script_engine.idle().await;
            assert!(
                unhandled_exceptions.is_empty(),
                "unhandled exceptions found: {unhandled_exceptions:?}"
            );

            slint::quit_event_loop().unwrap();

            Result::<()>::Ok(())
        }))?;

        slint::run_event_loop_until_quit()?;

        task_tracker.close();
        cancellation_token.cancel();

        tokio_runtime.block_on(handle).unwrap();
        tokio_runtime.block_on(task_tracker.wait());

        Ok(())
    }

    pub fn run<F>(f: F) -> Result<()>
    where
        F: AsyncFnOnce(Arc<Self>, &mut ScriptEngine) -> Result<()> + 'static,
    {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        tokio_runtime.block_on(async {
            let cancellation_token = CancellationToken::new();
            let task_tracker = TaskTracker::new();

            let local_cancellation_token = cancellation_token.clone();
            task_tracker.spawn(async move {
                select! {
                    _ = signal::ctrl_c() => {
                        local_cancellation_token.cancel();
                    },
                    _ = local_cancellation_token.cancelled() => {},
                }
            });

            let (runtime, mut script_engine) =
                Self::new(cancellation_token.clone(), task_tracker.clone()).await?;

            f(runtime, &mut script_engine).await?;

            task_tracker.close();
            cancellation_token.cancel();

            task_tracker.wait().await;

            Result::<()>::Ok(())
        })?;

        Ok(())
    }

    #[cfg(unix)]
    pub const fn platform(&self) -> &x11::Runtime {
        &self.runtime
    }

    #[cfg(windows)]
    pub fn platform(&self) -> &win::Runtime {
        &self.runtime
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub fn task_tracker(&self) -> TaskTracker {
        self.task_tracker.clone()
    }

    pub fn enigo(&self) -> Arc<Mutex<Enigo>> {
        self.enigo.clone()
    }

    #[inline]
    pub fn block_on<F: Future<Output = R>, R>(f: F) -> R {
        block_in_place(|| -> R { Handle::current().block_on(f) })
    }

    pub fn test<F>(f: F)
    where
        F: AsyncFnOnce(Arc<Self>) -> () + 'static,
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

    pub fn test_with_script_engine<F>(f: F)
    where
        F: AsyncFnOnce(&mut ScriptEngine) -> () + 'static,
    {
        Self::run(async move |_runtime, mut script_engine| {
            f(&mut script_engine).await;

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
    use convert_case::{Case, Casing};
    use derive_more::Display;
    use macros::ExposeEnum;
    use rquickjs::{Function, Object, Value, atom::PredefinedAtom, class::Trace};

    use super::*;

    fn print<'js>(value: Value<'js>) {
        println!("{value:?}");
    }

    #[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
    #[rquickjs::class]
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

    #[derive(JsLifetime, Trace)]
    #[rquickjs::class]
    pub struct TestSingletonStruct {
        string: String,
        integer: i64,
        float: f64,
    }

    #[rquickjs::methods(rename_all = "camelCase")]
    impl TestSingletonStruct {}

    impl Default for TestSingletonStruct {
        fn default() -> Self {
            Self {
                string: Default::default(),
                integer: Default::default(),
                float: Default::default(),
            }
        }
    }

    impl SingletonClass<'_> for TestSingletonStruct {}

    async fn setup(script_engine: &mut ScriptEngine) {
        script_engine
            .with(|ctx| {
                ctx.globals()
                    .prop("print", Function::new(ctx.clone(), print))
                    .unwrap();
                TestEnum::register(&ctx).unwrap();
                TestSingletonStruct::register(&ctx, TestSingletonStruct::default()).unwrap();
                TestGenerator::register(&ctx).unwrap();
            })
            .await;
    }

    #[test]
    fn test_enum() {
        Runtime::test_with_script_engine(async move |script_engine| {
            setup(script_engine).await;

            let result = script_engine.eval::<TestEnum>("TestEnum.B").await.unwrap();
            assert_eq!(result, TestEnum::B)
        });
    }

    #[test]
    fn test_singleton() {
        Runtime::test_with_script_engine(async move |script_engine| {
            setup(script_engine).await;

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
