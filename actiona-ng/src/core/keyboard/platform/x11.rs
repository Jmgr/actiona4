use std::sync::Arc;

use super::{Key, KeyboardImplTrait, Result};
use crate::runtime::Runtime;

#[derive(Debug)]
pub struct KeyboardImpl {
    _runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub const fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { _runtime: runtime })
    }
}

impl KeyboardImplTrait for KeyboardImpl {
    fn is_key_pressed(&self, _key: Key) -> Result<bool> {
        // TODO
        Ok(false)
    }
}
