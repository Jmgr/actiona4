use std::sync::{
    Arc, Weak,
    atomic::{AtomicUsize, Ordering},
};

use color_eyre::Result;
use derive_more::{Constructor, Deref, DerefMut};
use enigo::Key;
use itertools::Itertools;
use tokio::{
    select,
    sync::{broadcast, mpsc, watch},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;

use crate::{
    core::{
        mouse::Button,
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
    },
    types::input::Direction,
};

pub trait Signal<T>: Send + Sync + 'static {
    type Receiver;
    fn send(&self, value: T);
    fn subscribe(&self) -> Self::Receiver;
    fn new() -> Self;
}

#[derive(Clone, Debug)]
pub struct AllSignals<T>(broadcast::Sender<T>);

impl<T: Send + Sync + 'static> Signal<T> for AllSignals<T> {
    type Receiver = broadcast::Receiver<T>;
    fn send(&self, value: T) {
        _ = self.0.send(value);
    }
    fn subscribe(&self) -> Self::Receiver {
        self.0.subscribe()
    }
    fn new() -> Self {
        Self(broadcast::Sender::new(1024)) // TODO
    }
}

#[derive(Clone, Debug)]
pub struct LatestOnlySignals<T>(watch::Sender<T>);

impl<T: Send + Sync + Default + 'static> Signal<T> for LatestOnlySignals<T> {
    type Receiver = watch::Receiver<T>;
    fn send(&self, value: T) {
        _ = self.0.send(value);
    }
    fn subscribe(&self) -> Self::Receiver {
        self.0.subscribe()
    }
    fn new() -> Self {
        Self(watch::Sender::new(T::default())) // TODO
    }
}

pub trait Topic: Send + Sync + 'static {
    type T;
    type Signal: Signal<Self::T> + Clone;

    fn on_start(&self) -> impl Future<Output = Result<()>> + Send;
    fn on_stop(&self) -> impl Future<Output = Result<()>> + Send;
}

#[derive(Debug)]
pub struct Guard<T: Topic> {
    topic_wrapper: Arc<TopicWrapper<T>>,
    signal_sender: T::Signal, // TODO: use a receiver instead
}

impl<T: Topic> Drop for Guard<T> {
    fn drop(&mut self) {
        self.topic_wrapper.decrement();
    }
}

impl<T: Topic> Guard<T> {
    pub fn subscribe(&self) -> <T::Signal as Signal<T::T>>::Receiver {
        self.signal_sender.subscribe()
    }
}

enum SubscribersChange {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub struct TopicWrapper<T: Topic> {
    signal_sender: T::Signal,
    subscribers_change_sender: mpsc::UnboundedSender<SubscribersChange>,
    topic: Arc<T>,
}

impl<T: Topic + 'static> TopicWrapper<T> {
    pub fn new(topic: T, cancellation_token: CancellationToken, task_tracker: TaskTracker) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let topic = Arc::new(topic);

        let local_topic = topic.clone();
        task_tracker.spawn(async move {
            let mut count: usize = 0;
            loop {
                let command = select! {
                    _ = cancellation_token.cancelled() => { break; }
                    command = receiver.recv() => { command }
                };

                let Some(command) = command else {
                    break;
                };

                match command {
                    SubscribersChange::Increment => {
                        if count == 0
                            && let Err(err) = local_topic.on_start().await
                        {
                            error!("{}", err); // TODO: improve this
                        }

                        count += 1;
                    }
                    SubscribersChange::Decrement => {
                        if count == 1
                            && let Err(err) = local_topic.on_stop().await
                        {
                            error!("{}", err);
                        }

                        count -= 1;
                    }
                }
            }
        });

        Self {
            signal_sender: T::Signal::new(),
            subscribers_change_sender: sender,
            topic,
        }
    }

    #[must_use]
    pub fn subscribe(self: &Arc<Self>) -> Guard<T> {
        self.increment();

        Guard {
            topic_wrapper: self.clone(),
            signal_sender: self.signal_sender.clone(),
        }
    }

    pub fn publish(&self, value: T::T) {
        self.signal_sender.send(value);
    }

    fn increment(&self) {
        _ = self
            .subscribers_change_sender
            .send(SubscribersChange::Increment);
    }

    fn decrement(&self) {
        _ = self
            .subscribers_change_sender
            .send(SubscribersChange::Decrement);
    }

    pub fn topic(&self) -> Arc<T> {
        self.topic.clone()
    }
}

// TODO: remove
pub struct Test {
    parent: Weak<MultiTest>,
}

impl Topic for Test {
    type T = u32;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_start().await?;
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_stop().await?;
        }
        Ok(())
    }
}

pub struct Test2 {
    parent: Weak<MultiTest>,
}

impl Topic for Test2 {
    type T = u64;
    type Signal = LatestOnlySignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_start().await?;
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_stop().await?;
        }
        Ok(())
    }
}

pub struct MultiTest {
    test: Arc<TopicWrapper<Test>>,
    test2: Arc<TopicWrapper<Test2>>,
    subscribers: Arc<AtomicUsize>,
}

impl MultiTest {
    #[must_use]
    pub fn new(cancellation_token: CancellationToken, task_tracker: TaskTracker) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            test: Arc::new(TopicWrapper::new(
                Test { parent: me.clone() },
                cancellation_token.clone(),
                task_tracker.clone(),
            )),
            test2: Arc::new(TopicWrapper::new(
                Test2 { parent: me.clone() },
                cancellation_token.clone(),
                task_tracker,
            )),
            subscribers: Arc::new(AtomicUsize::new(0)),
        })
    }

    #[must_use]
    pub fn subscribe_test(&self) -> Guard<Test> {
        self.test.subscribe()
    }

    #[must_use]
    pub fn subscribe_test2(&self) -> Guard<Test2> {
        self.test2.subscribe()
    }

    pub fn publish_test(&self, value: <Test as Topic>::T) {
        self.test.publish(value);
    }

    pub fn publish_test2(&self, value: <Test2 as Topic>::T) {
        self.test2.publish(value);
    }

    async fn on_start(&self) -> Result<()> {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            println!("MultiTest start");
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if self.subscribers.fetch_sub(1, Ordering::Relaxed) == 1 {
            println!("MultiTest stop");
        }
        Ok(())
    }
}

#[derive(Clone, Constructor, Debug)]
pub struct MouseButtonEvent {
    pub button: Button,
    pub direction: Direction,
    pub is_injected: bool,
}

#[derive(Clone, Constructor, Debug, Default)]
pub struct MouseMoveEvent {
    pub position: Point,
    pub is_injected: bool,
}

#[derive(Clone, Constructor, Debug)]
pub struct KeyboardKeyEvent {
    pub key: Key,
    pub scan_code: u32,
    pub direction: Direction,
    pub is_injected: bool,
    pub name: String,
    pub is_repeat: bool,
}

#[derive(Clone, Constructor, Debug)]
pub struct KeyboardTextEvent {
    pub character: char,
    pub is_injected: bool,
    pub is_repeat: bool,
}

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
            rect: rect(point(value.x, value.y), size(value.width, value.height)),
            width_mm: value.width_mm,
            height_mm: value.height_mm,
            rotation: value.rotation,
            scale_factor: value.scale_factor,
            frequency: value.frequency,
            is_primary: value.is_primary,
        }
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct DisplayInfoVec(pub Vec<DisplayInfo>);

impl From<Vec<display_info::DisplayInfo>> for DisplayInfoVec {
    fn from(value: Vec<display_info::DisplayInfo>) -> Self {
        Self(value.iter().cloned().map(|info| info.into()).collect_vec())
    }
}

#[derive(Clone, Debug)]
pub struct WindowEvent {
    pub window: u32,
    pub status: WindowStatus,
}

#[derive(Clone, Debug)]
pub enum WindowStatus {
    Closed,
    Visible,
    Hidden,
}
