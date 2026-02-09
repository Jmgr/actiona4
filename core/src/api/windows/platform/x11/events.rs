use crate::api::windows::platform::x11::WindowHandle;

#[derive(Clone, Debug)]
pub enum WindowEvent {
    Closed(WindowHandle),
    Hidden(WindowHandle),
    Visible(WindowHandle),
}
