use std::{collections::HashSet, hash::Hash};

use bimap::BiMap;

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
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
    Other(eyre::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait WindowsHandler {
    fn all_windows(&mut self) -> Result<Vec<WindowId>>;
    fn is_window_visible(&self, id: WindowId) -> Result<bool>;
    fn window_title(&self, id: WindowId) -> Result<String>;
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
    pub fn update(&mut self, handles: impl IntoIterator<Item = H>, len: usize) -> Vec<WindowId> {
        let mut window_ids = Vec::with_capacity(len);
        let mut existing_handles = HashSet::with_capacity(len);

        for window_handle in handles.into_iter() {
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
}
