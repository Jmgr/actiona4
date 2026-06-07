use macros::rpc_protocol;
use types::{Point, Rect};

#[rpc_protocol]
#[derive(Debug)]
pub trait SelectionProtocol {
    #[host_call]
    async fn select_rect() -> Option<Rect>;

    #[host_call]
    async fn select_position() -> Option<Point>;
}
