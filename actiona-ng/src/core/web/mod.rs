pub mod js;

#[derive(Clone, Debug)]
pub struct Web {
    inner: reqwest::Client,
}

impl Web {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }
}
