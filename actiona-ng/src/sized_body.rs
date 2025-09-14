use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub struct SizedBody<B> {
    inner: B,
    size: u64,
}

impl<B> SizedBody<B> {
    pub fn new(inner: B, size: u64) -> Self {
        Self { inner, size }
    }
}

impl<B> Body for SizedBody<B>
where
    B: Body<Data = Bytes, Error = BoxError> + Unpin,
{
    type Data = Bytes;
    type Error = B::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.inner).poll_frame(cx)
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.size)
    }
}
