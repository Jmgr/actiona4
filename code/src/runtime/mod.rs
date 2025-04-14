use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use async_compat::Compat;
use enigo::{Enigo, Settings};
use eyre::Result;
use itertools::Itertools;
use rquickjs::{Context as JsContext, JsLifetime, Runtime as JsRuntime};
use tokio::{
    runtime::Handle,
    select, signal,
    sync::broadcast::{self, Receiver, Sender},
    task::block_in_place,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::core::{
    SingletonClass, ValueClass,
    color::js::JsColor,
    console::js::JsConsole,
    displays::{
        Displays,
        js::JsDisplays,
    },
    file::js::JsFile,
    image::js::JsImage,
    keyboard::js::JsKeyboard,
    mouse::{JsButton, js::JsMouse},
    name::js::{JsName, JsWildcard},
    point::js::JsPoint,
    rect::{Rect, js::JsRect, rect},
    screenshot::js::JsScreenshot,
    ui::js::JsUi,
};

pub mod platform;

#[cfg(windows)]
use platform::win;
#[cfg(unix)]
use platform::x11;

// This is the same as display_info::DisplayInfo, but without the pointer to the raw monitor handle, since it is not Send.
#[derive(Clone, Debug)]
pub struct DisplayInfo {
    /// Unique identifier associated with the display.
    pub id: u32,
    /// The display name
    pub name: String,
    /// The display friendly name
    pub friendly_name: String,
    /// The display pixel rectangle.
    pub rect: Rect,
    /// The width of a display in millimeters. This value may be 0.
    pub width_mm: i32,
    /// The height of a display in millimeters. This value may be 0.
    pub height_mm: i32,
    /// Can be 0, 90, 180, 270, represents screen rotation in clock-wise degrees.
    pub rotation: f32,
    /// Output device's pixel scale factor.
    pub scale_factor: f32,
    /// The display refresh rate.
    pub frequency: f32,
    /// Whether the screen is the main screen
    pub is_primary: bool,
}

impl From<display_info::DisplayInfo> for DisplayInfo {
    fn from(value: display_info::DisplayInfo) -> Self {
        Self {
            id: value.id,
            name: value.name,
            friendly_name: value.friendly_name,
            rect: rect(value.x, value.y, value.width, value.height),
            width_mm: value.width_mm,
            height_mm: value.height_mm,
            rotation: value.rotation,
            scale_factor: value.scale_factor,
            frequency: value.frequency,
            is_primary: value.is_primary,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayInfoVec(pub Vec<DisplayInfo>);

impl Deref for DisplayInfoVec {
    type Target = Vec<DisplayInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DisplayInfoVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<display_info::DisplayInfo>> for DisplayInfoVec {
    fn from(value: Vec<display_info::DisplayInfo>) -> Self {
        Self(value.iter().cloned().map(|info| info.into()).collect_vec())
    }
}

#[derive(Clone, Debug)]
pub enum Direction {
    Pressed,
    Released,
}

#[derive(Clone, Debug)]
pub enum RecordEvent {
    MouseButton(JsButton, Direction),
    DisplayChanged(DisplayInfoVec),
}

#[derive(Debug, JsLifetime)]
pub(crate) struct JsUserData {
    displays: Arc<Displays>,
}

impl JsUserData {
    fn new(displays: Arc<Displays>) -> Self {
        Self { displays }
    }

    pub(crate) fn displays(&self) -> Arc<Displays> {
        self.displays.clone()
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
    events_sender: Sender<RecordEvent>,
}

impl Runtime {
    // TODO: make private
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<(Arc<Self>, JsContext)> {
        let (events_sender, _) = broadcast::channel(128);

        #[cfg(unix)]
        let runtime = x11::Runtime::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            events_sender.clone(),
        )
        .await?;

        #[cfg(windows)]
        let runtime = win::Runtime::new(
            cancellation_token.clone(),
            task_tracker.clone(),
            events_sender.clone(),
        )
        .await?;

        let js_runtime = JsRuntime::new()?;
        let js_context = JsContext::full(&js_runtime)?;

        let runtime = Arc::new(Self {
            runtime,
            enigo: Arc::new(Mutex::new(Enigo::new(&Settings::default())?)),
            cancellation_token: cancellation_token.clone(),
            task_tracker: task_tracker.clone(),
            events_sender,
        });

        let displays = Arc::new(Displays::new(runtime.clone())?);

        let mouse = JsMouse::new(runtime.clone()).await?;
        let keyboard = JsKeyboard::new(runtime.clone())?;
        let ui = JsUi::new(runtime.clone(), displays.clone())?;
        let console = JsConsole::new(runtime.clone())?;
        let js_displays = JsDisplays::new(displays.clone())?;
        let screenshot = JsScreenshot::new(runtime.clone(), displays.clone()).await?;

        js_context.with(|ctx| -> Result<()> {
            ctx.store_userdata(JsUserData::new(displays)).unwrap();

            // Value classes
            JsPoint::register(&ctx)?;
            JsRect::register(&ctx)?;
            JsColor::register(&ctx)?;
            JsImage::register(&ctx)?;
            JsFile::register(&ctx)?;
            JsWildcard::register(&ctx)?;
            JsName::register(&ctx)?;

            // Singletons
            JsMouse::register(&ctx, mouse)?;
            JsKeyboard::register(&ctx, keyboard)?;
            JsUi::register(&ctx, ui)?;
            JsConsole::register(&ctx, console)?;
            JsDisplays::register(&ctx, js_displays)?;
            JsScreenshot::register(&ctx, screenshot)?;

            Ok(())
        })?;

        Ok((runtime, js_context))
    }

    pub fn run<F, Fut>(f: F) -> Result<()>
    where
        F: FnOnce(Arc<Self>, JsContext) -> Fut + 'static,
        Fut: Future<Output = Result<()>>,
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
            let (runtime, js_context) = Runtime::new(local_cancellation_token, local_task_tracker)
                .await
                .unwrap();

            f(runtime, js_context).await.unwrap();

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

    pub fn run_without_ui<F, Fut>(f: F) -> Result<()>
    where
        F: FnOnce(Arc<Self>, JsContext) -> Fut + 'static,
        Fut: Future<Output = Result<()>>,
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

            let (runtime, js_context) =
                Runtime::new(cancellation_token.clone(), task_tracker.clone())
                    .await
                    .unwrap();

            f(runtime, js_context).await.unwrap();

            task_tracker.close();
            cancellation_token.cancel();

            task_tracker.wait().await;
        });

        Ok(())
    }

    #[cfg(unix)]
    pub fn platform(&self) -> &x11::Runtime {
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

    pub fn subcribe_events(&self) -> Receiver<RecordEvent> {
        self.events_sender.subscribe()
    }

    pub fn enigo(&self) -> Arc<Mutex<Enigo>> {
        self.enigo.clone()
    }

    #[inline]
    pub fn block_on<F: Future<Output = R>, R>(f: F) -> R {
        block_in_place(|| -> R { Handle::current().block_on(f) })
    }

    pub fn test<F, Fut>(f: F)
    where
        F: FnOnce(Arc<Self>) -> Fut + 'static,
        Fut: Future,
    {
        Self::run_without_ui(|runtime, _js_context| async {
            f(runtime).await;
            Ok(())
        })
        .unwrap();
    }

    pub fn test_with_js<F, Fut>(f: F)
    where
        F: FnOnce(JsContext) -> Fut + 'static,
        Fut: Future,
    {
        Self::run_without_ui(|_runtime, js_context| async {
            f(js_context).await;
            Ok(())
        })
        .unwrap();
    }
}
