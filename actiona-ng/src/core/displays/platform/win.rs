use std::sync::Arc;

use super::Result;
use crate::runtime::Runtime;

#[derive(Clone, Debug, Default)]
pub struct DisplaysImpl;

impl DisplaysImpl {
    pub fn new(_runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self)
    }
}
