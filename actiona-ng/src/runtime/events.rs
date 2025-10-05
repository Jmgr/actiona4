use std::{
    marker::PhantomData,
    sync::{
        Arc,
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

impl<T: Send + Sync + 'static> Signal<T> for LatestOnlySignals<T> {
    type Receiver = watch::Receiver<T>;
    fn send(&self, value: T) {
        let _ = self.sender.send(value);
    }
    fn subscribe(&self) -> Self::Receiver {
        self.sender.subscribe()
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
    signal: S,
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
                signal,
                control_sender,
                subscribers,
                phantom: PhantomData::default(),
            },
            control_receiver,
        )
    }

    pub fn publish(&self, value: T) {
        let _ = self.signal.send(value);
    }

    pub fn subscribe(&self) -> ReceiverGuard<T, S> {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            let _ = self.control_sender.send_replace(Control::Enable);
        }

        ReceiverGuard {
            signal: self.signal.clone(),
            subscribers: self.subscribers.clone(),
            control_sender: self.control_sender.clone(),
            phantom: PhantomData::default(),
        }
    }
}

#[derive(Debug)] // No clone
pub struct ReceiverGuard<T, S: Signal<T>> {
    signal: S,
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
        self.signal.subscribe()
    }
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
