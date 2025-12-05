use super::Result;

#[derive(Debug)]
pub struct DisplaysImpl {} // TODO: needed?

impl DisplaysImpl {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
