#![cfg_attr(windows, windows_subsystem = "console")]

use actiona_common::sentry::setup_crash_reporting;
use clap::Parser;
use color_eyre::Result;
use winit::event_loop::EventLoop;

use crate::{app::App, cli::Cli, events::AppEvent, screenshot::capture_screenshot};

mod app;
mod cli;
#[cfg(not(windows))]
mod cursor_tracker;
mod events;
mod magnifier;
mod screenshot;
mod text;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() -> Result<()> {
    let _guard = setup_crash_reporting(built_info::PKG_NAME)?;

    let cli = Cli::parse();
    let screenshot = Some(capture_screenshot());

    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let mut app = App::new(cli.mode, screenshot, proxy);
    event_loop.run_app(&mut app)?;

    Ok(())
}
