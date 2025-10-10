use std::{collections::HashSet, hash::Hash};

use bimap::BiMap;

use crate::core::{point::Point, rect::Rect, size::Size};

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct WindowId(u64);

impl WindowId {
    pub(crate) fn next(&mut self) -> Self {
        self.0 = self.0.wrapping_add(1);

        *self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported")]
    Unsupported,

    #[error("not found")]
    NotFound,

    #[error(transparent)]
    Other(#[from] eyre::Report),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait WindowsHandler {
    fn all(&mut self) -> Result<Vec<WindowId>>;
    fn is_visible(&self, id: WindowId) -> Result<bool>;
    fn title(&self, id: WindowId) -> Result<String>;
    fn classname(&self, id: WindowId) -> Result<String>;
    fn close(&self, id: WindowId) -> Result<()>;
    fn process_id(&self, id: WindowId) -> Result<u32>;
    fn rect(&self, id: WindowId) -> Result<Rect>;
    fn set_active(&self, id: WindowId) -> Result<()>;
    fn minimize(&self, id: WindowId) -> Result<()>;
    fn maximize(&self, id: WindowId) -> Result<()>;
    fn set_position(&self, id: WindowId, position: Point) -> Result<()>;
    fn position(&self, id: WindowId) -> Result<Point>;
    fn set_size(&self, id: WindowId, size: Size) -> Result<()>;
    fn size(&self, id: WindowId) -> Result<Size>;
    fn is_active(&self, id: WindowId) -> Result<bool>;
    fn active_window(&mut self) -> Result<WindowId>;
}

#[derive(Debug)]
pub struct Registry<H: Clone + Eq + Hash> {
    map: BiMap<WindowId, H>,
    next_id: WindowId,
}

impl<H: Clone + Eq + Hash> Default for Registry<H> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            next_id: Default::default(),
        }
    }
}

impl<H: Clone + Eq + Hash> Registry<H> {
    pub fn update(&mut self, handles: impl IntoIterator<Item = H>) -> Vec<WindowId> {
        let iter = handles.into_iter();
        let (lower, _upper) = iter.size_hint();
        let mut window_ids = Vec::with_capacity(lower);
        let mut existing_handles = HashSet::with_capacity(lower);

        for window_handle in iter {
            existing_handles.insert(window_handle.clone());

            if let Some(existing) = self.map.get_by_right(&window_handle).copied() {
                window_ids.push(existing);
            } else {
                let next_id = self.next_id.next();
                window_ids.push(next_id);
                self.map.insert(next_id, window_handle);
            }
        }

        self.map
            .retain(|_, window_handle| existing_handles.contains(&window_handle));

        window_ids
    }

    pub fn get_handle(&self, id: WindowId) -> Result<&H> {
        self.map.get_by_left(&id).ok_or(Error::NotFound)
    }

    pub fn get_id(&self, handle: &H) -> Option<WindowId> {
        self.map.get_by_right(handle).copied()
    }

    pub fn contains_id(&self, id: WindowId) -> bool {
        self.map.contains_left(&id)
    }

    pub fn contains_handle(&self, handle: &H) -> bool {
        self.map.contains_right(&handle)
    }
}
