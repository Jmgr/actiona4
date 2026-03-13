#![cfg_attr(windows, windows_subsystem = "console")]

mod app;
mod cli;
#[cfg(not(windows))]
mod cursor_tracker;
mod events;
mod magnifier;
mod screenshot;
mod text;

use clap::Parser;
use winit::event_loop::EventLoop;

use crate::{app::App, cli::Cli, events::AppEvent, screenshot::capture_screenshot};

fn main() {
    let cli = Cli::parse();
    let screenshot = Some(capture_screenshot());

    let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();
    let mut app = App::new(cli.mode, screenshot, proxy);
    event_loop.run_app(&mut app).unwrap();
}
