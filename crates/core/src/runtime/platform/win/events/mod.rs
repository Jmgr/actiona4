pub mod input;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowHandle(pub isize);

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Closed(WindowHandle),
}
