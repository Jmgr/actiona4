#![cfg_attr(windows, windows_subsystem = "console")]

use std::time::Duration;

use actiona_common::sentry::setup_crash_reporting;
use color_eyre::{
    Result,
    eyre::{OptionExt, eyre},
};
use extension::{
    Extension,
    protocols::selection::{SelectionProtocol, SelectionProtocolExtension},
};
use tokio::sync::oneshot;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use types::{point::Point, rect::Rect};
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::{app::App, events::AppEvent, screenshot::capture_screenshot};

mod app;
#[cfg(not(windows))]
mod cursor_tracker;
mod events;
mod magnifier;
mod screenshot;
mod text;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

struct SelectionExtension {
    proxy: EventLoopProxy<AppEvent>,
}

impl SelectionProtocolExtension for SelectionExtension {
    async fn select_rect(&self) -> Result<Option<Rect>> {
        let screenshot = capture_screenshot().await?;
        let (response, selection) = oneshot::channel();
        self.proxy
            .send_event(AppEvent::SelectRect {
                screenshot,
                response,
            })
            .map_err(|_| eyre!("selection event loop is not available"))?;
        selection
            .await
            .map_err(|_| eyre!("selection request was cancelled"))
    }

    async fn select_position(&self) -> Result<Option<Point>> {
        let screenshot = capture_screenshot().await?;
        let (response, selection) = oneshot::channel();
        self.proxy
            .send_event(AppEvent::SelectPosition {
                screenshot,
                response,
            })
            .map_err(|_| eyre!("selection event loop is not available"))?;
        selection
            .await
            .map_err(|_| eyre!("selection request was cancelled"))
    }
}

fn main() -> Result<()> {
    let _guard = setup_crash_reporting(built_info::PKG_NAME)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    let extension = Extension::<SelectionProtocol>::with_handler_impl(
        std::env::args()
            .nth(1)
            .ok_or_eyre("expected a key")?
            .try_into()
            .map_err(|_| eyre!("expected a valid RPC connection key"))?,
        task_tracker.clone(),
        cancellation_token.clone(),
        Duration::from_secs(60),
        SelectionExtension {
            proxy: proxy.clone(),
        },
    );

    let extension_task = runtime.spawn(async move { extension.run().await });
    let shutdown_proxy = proxy.clone();
    let shutdown_token = cancellation_token.clone();
    let shutdown_task = runtime.spawn(async move {
        match extension_task.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("selection extension stopped: {error:?}"),
            Err(error) => eprintln!("selection extension task failed: {error}"),
        }
        shutdown_token.cancel();
        let _ = shutdown_proxy.send_event(AppEvent::Shutdown);
    });

    let mut app = App::new(proxy);
    event_loop.run_app(&mut app)?;

    cancellation_token.cancel();
    task_tracker.close();
    runtime.block_on(async {
        let _ = shutdown_task.await;
        task_tracker.wait().await;
    });

    Ok(())
}
