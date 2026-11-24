#![cfg_attr(windows, windows_subsystem = "windows")]

use actiona_common::sentry::setup_crash_reporting;
use color_eyre::Result;
use gpui_kit::{application, assets::Assets, init};

#[allow(clippy::needless_raw_strings)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() -> Result<()> {
    let _guard = setup_crash_reporting(built_info::PKG_NAME)?;

    application().with_assets(Assets).run(|cx| {
        init(cx);

        // Quit once the event loop is running. There is no editor window yet.
        cx.spawn(async |cx| cx.update(|app| app.quit())).detach();
    });

    Ok(())
}
