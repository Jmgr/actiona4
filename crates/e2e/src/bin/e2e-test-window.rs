//! Opens an empty window for the end-to-end tests to manipulate, titled after the first argument
//! so that tests can find it. Exits when the window is closed.

use std::env;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct TestWindow {
    title: String,
    window: Option<Window>,
}

impl ApplicationHandler for TestWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(PhysicalSize::new(300, 200));
        self.window = Some(
            event_loop
                .create_window(attributes)
                .expect("failed to create the test window"),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    let title = env::args()
        .nth(1)
        .unwrap_or_else(|| "actiona4 e2e test window".to_owned());

    EventLoop::new()?.run_app(&mut TestWindow {
        title,
        window: None,
    })
}
