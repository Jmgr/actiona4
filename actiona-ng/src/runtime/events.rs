use std::{
    marker::PhantomData,
    sync::{
        Arc, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

use derive_more::{Deref, DerefMut, Display};
use enigo::Direction;
use itertools::Itertools;
use tokio::sync::{broadcast, watch};

use crate::core::{
    mouse::Button,
    rect::{Rect, rect},
};

pub trait Signal<T>: Send + Sync + 'static {
    type Receiver;
    fn send(&self, value: T);
    fn subscribe(&self) -> Self::Receiver;
    fn new() -> Self;
}

#[derive(Clone, Debug)]
pub struct AllSignals<T> {
    sender: broadcast::Sender<T>,
}

impl<T: Send + Sync + 'static> Signal<T> for AllSignals<T> {
    type Receiver = broadcast::Receiver<T>;
    fn send(&self, value: T) {
        let _ = self.sender.send(value);
    }
    fn subscribe(&self) -> Self::Receiver {
        self.sender.subscribe()
    }
    fn new() -> Self {
        Self {
            sender: broadcast::Sender::new(1024), // TODO
        }
    }
}

impl<T: Clone + Send + Sync + 'static> AllSignals<T> {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024); // TODO
        Self { sender }
    }
}

#[derive(Clone, Debug)]
pub struct LatestOnlySignals<T> {
    sender: watch::Sender<T>,
}

impl<T: Send + Sync + Default + 'static> Signal<T> for LatestOnlySignals<T> {
    type Receiver = watch::Receiver<T>;
    fn send(&self, value: T) {
        let _ = self.sender.send(value);
    }
    fn subscribe(&self) -> Self::Receiver {
        self.sender.subscribe()
    }
    fn new() -> Self {
        Self {
            sender: watch::Sender::new(T::default()), // TODO
        }
    }
}

impl<T: Clone + Send + Sync + Default + 'static> LatestOnlySignals<T> {
    pub fn new() -> Self {
        let (sender, _) = watch::channel(T::default());
        Self { sender }
    }
}

#[derive(Clone, Copy, Debug, Default, Display)]
pub enum Control {
    Enable,

    #[default]
    Disable,
}

#[derive(Debug)]
pub struct Topic<T, S: Signal<T>> {
    signal_sender: S,
    control_sender: watch::Sender<Control>,
    subscribers: Arc<AtomicUsize>,
    phantom: PhantomData<T>,
}

impl<T: Clone + Send + 'static, S: Signal<T> + Clone> Topic<T, S> {
    pub fn new(signal: S) -> (Self, watch::Receiver<Control>) {
        let (control_sender, control_receiver) = watch::channel(Control::default());
        let subscribers = Arc::new(AtomicUsize::new(0));

        (
            Self {
                signal_sender: signal,
                control_sender,
                subscribers,
                phantom: PhantomData::default(),
            },
            control_receiver,
        )
    }

    pub fn publish(&self, value: T) {
        let _ = self.signal_sender.send(value);
    }

    pub fn subscribe(&self) -> ReceiverGuard<T, S> {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            let _ = self.control_sender.send_replace(Control::Enable);
        }

        ReceiverGuard {
            signal_sender: self.signal_sender.clone(),
            subscribers: self.subscribers.clone(),
            control_sender: self.control_sender.clone(),
            phantom: PhantomData::default(),
        }
    }
}

#[derive(Debug)] // No clone
pub struct ReceiverGuard<T, S: Signal<T>> {
    signal_sender: S,
    subscribers: Arc<AtomicUsize>,
    control_sender: watch::Sender<Control>,
    phantom: PhantomData<T>,
}

impl<T, S: Signal<T>> Drop for ReceiverGuard<T, S> {
    fn drop(&mut self) {
        if self.subscribers.fetch_sub(1, Ordering::Relaxed) == 1 {
            let _ = self.control_sender.send_replace(Control::Disable);
        }
    }
}

impl<T: Clone, S: Signal<T>> ReceiverGuard<T, S> {
    pub fn subscribe(&self) -> S::Receiver {
        self.signal_sender.subscribe()
    }
}

///

pub trait TopicImpl {
    type T;
    type Signal: Signal<Self::T> + Clone;

    fn on_start(&self);
    fn on_stop(&self);
}

pub struct Guard<I: TopicImpl> {
    topic2: Arc<Topic2<I>>,
    signal_sender: I::Signal,
}

impl<I: TopicImpl> Drop for Guard<I> {
    fn drop(&mut self) {
        self.topic2.decrement();
    }
}

impl<I: TopicImpl> Guard<I> {
    pub fn subscribe(&self) -> <I::Signal as Signal<I::T>>::Receiver {
        self.signal_sender.subscribe()
    }
}

pub struct Topic2<I: TopicImpl> {
    subscribers: Arc<AtomicUsize>,
    topic_impl: Arc<I>,
    signal_sender: I::Signal,
}

impl<I: TopicImpl> Topic2<I> {
    pub fn new(topic_impl: I) -> Self {
        Self {
            subscribers: Arc::new(AtomicUsize::new(0)),
            topic_impl: Arc::new(topic_impl),
            signal_sender: I::Signal::new(),
        }
    }

    pub fn subscribe(self: Arc<Self>) -> Guard<I> {
        self.increment();

        Guard {
            topic2: self.clone(),
            signal_sender: self.signal_sender.clone(),
        }
    }

    pub fn publish(&self, value: I::T) {
        let _ = self.signal_sender.send(value);
    }

    fn increment(&self) {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            self.topic_impl.on_start();
        }
    }

    fn decrement(&self) {
        if self.subscribers.fetch_sub(1, Ordering::Relaxed) == 1 {
            self.topic_impl.on_stop();
        }
    }
}

pub struct Test {
    parent: Weak<MultiTest>,
}

impl TopicImpl for Test {
    type T = u32;
    type Signal = AllSignals<Self::T>;

    fn on_start(&self) {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_start();
        }
    }

    fn on_stop(&self) {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_stop();
        }
    }
}

pub struct Test2 {
    parent: Weak<MultiTest>,
}

impl TopicImpl for Test2 {
    type T = u64;
    type Signal = LatestOnlySignals<Self::T>;

    fn on_start(&self) {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_start();
        }
    }

    fn on_stop(&self) {
        if let Some(parent) = self.parent.upgrade() {
            parent.on_stop();
        }
    }
}

pub struct MultiTest {
    test: Arc<Topic2<Test>>,
    test2: Arc<Topic2<Test2>>,
    subscribers: Arc<AtomicUsize>,
}

impl MultiTest {
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            test: Arc::new(Topic2::new(Test { parent: me.clone() })),
            test2: Arc::new(Topic2::new(Test2 { parent: me.clone() })),
            subscribers: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub fn subscribe_test(&self) -> Guard<Test> {
        self.test.clone().subscribe()
    }

    pub fn subscribe_test2(&self) -> Guard<Test2> {
        self.test2.clone().subscribe()
    }

    pub fn publish_test(&self, value: <Test as TopicImpl>::T) {
        self.test.publish(value);
    }

    pub fn publish_test2(&self, value: <Test2 as TopicImpl>::T) {
        self.test2.publish(value);
    }

    fn on_start(&self) {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            println!("MultiTest start");
        }
    }

    fn on_stop(&self) {
        if self.subscribers.fetch_sub(1, Ordering::Relaxed) == 1 {
            println!("MultiTest stop");
        }
    }
}

fn test() {
    let t = MultiTest::new();
    let _guard = t.subscribe_test();
}

#[derive(Clone, Debug)]
pub struct MouseButtonEvent {
    pub button: Button,
    pub direction: Direction,
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
