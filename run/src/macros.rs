use std::{collections::HashSet, path::Path, str::FromStr, sync::Arc, time::Duration};

use actiona_core::{
    api::{
        keyboard::js::{JsKey, JsStandardKey},
        macros::{MacroData, PlayConfig, PlayProgress, RecordConfig, play_impl, record_impl},
        mouse::Mouse,
    },
    runtime::{Runtime, WaitAtEnd},
};
use color_eyre::{
    Result,
    eyre::{bail, eyre},
};
use tokio::sync::watch;

fn parse_stop_key(s: &str) -> Result<enigo::Key> {
    let js_standard = JsStandardKey::from_str(s).map_err(|_| {
        eyre!(
            "unknown key name: {s:?}. Use standard key names like: Escape, Control, Alt, Shift, Space, Return, Tab, etc."
        )
    })?;
    enigo::Key::try_from(JsKey::Standard(js_standard))
        .map_err(|_| eyre!("key {s:?} is not supported on this platform"))
}

pub async fn run_record(
    runtime: Arc<Runtime>,
    file: &Path,
    stop_keys: &[String],
    timeout: Option<&str>,
    mouse_position_interval: &str,
    mouse_buttons: bool,
    mouse_position: bool,
    mouse_scroll: bool,
    keyboard_keys: bool,
) -> Result<()> {
    runtime.require_not_wayland()?;

    let enigo_stop_keys: HashSet<enigo::Key> = stop_keys
        .iter()
        .map(|s| parse_stop_key(s))
        .collect::<Result<_>>()?;

    let timeout = timeout.map(humantime::parse_duration).transpose()?;
    let mouse_position_interval = humantime::parse_duration(mouse_position_interval)?;

    let stop_display = stop_keys.join(" + ");
    println!("Recording... Press {stop_display} to stop.");
    if let Some(dur) = timeout {
        println!("(timeout: {})", humantime::format_duration(dur));
    }

    let config = RecordConfig {
        stop_keys: enigo_stop_keys,
        timeout,
        mouse_position_interval,
        mouse_buttons,
        mouse_position,
        mouse_scroll,
        keyboard_keys,
    };

    let mouse = Mouse::new(runtime.clone()).await?;
    let token = runtime.cancellation_token();

    let data = record_impl(runtime.clone(), mouse, config, runtime.displays(), token).await?;

    let duration = Duration::from_millis(data.metadata.duration_ms);
    println!(
        "Recorded {} events ({}).",
        data.events.len(),
        humantime::format_duration(duration)
    );

    data.save(file, runtime.task_tracker()).await?;
    println!("Saved to {}", file.display());

    runtime.set_wait_at_end(WaitAtEnd::No);
    Ok(())
}

pub async fn run_play(
    runtime: Arc<Runtime>,
    file: &Path,
    speed: f64,
    mouse_buttons: bool,
    mouse_position: bool,
    relative_mouse_position: bool,
    mouse_scroll: bool,
    keyboard_keys: bool,
) -> Result<()> {
    runtime.require_not_wayland()?;

    if speed <= 0.0 {
        bail!("speed must be greater than zero");
    }

    let data = MacroData::load(file, runtime.task_tracker()).await?;

    let duration = Duration::from_millis(data.metadata.duration_ms);
    println!(
        "Loaded macro: {} events, {}.",
        data.events.len(),
        humantime::format_duration(duration)
    );
    println!("Replaying at {speed}x speed...");

    let mouse = Mouse::new(runtime.clone()).await?;
    let token = runtime.cancellation_token();

    let config = PlayConfig {
        speed,
        mouse_buttons,
        mouse_position,
        relative_mouse_position,
        mouse_scroll,
        keyboard_keys,
    };

    let total = u64::try_from(data.events.len()).unwrap_or(u64::MAX);
    let (progress_tx, mut progress_rx) = watch::channel(PlayProgress::default());

    let progress_handle = runtime.task_tracker().spawn(async move {
        loop {
            if progress_rx.changed().await.is_err() {
                break;
            }
            let p = *progress_rx.borrow();
            if total > 0 {
                let pct = p.events_done * 100 / total;
                eprint!(
                    "\rPlaying: {}/{} events ({}%)   ",
                    p.events_done, total, pct
                );
            }
        }
        println!();
    });

    play_impl(
        runtime.clone(),
        mouse,
        &data,
        &config,
        runtime.displays(),
        progress_tx,
        token,
    )
    .await?;

    let _ = progress_handle.await;
    println!("Done.");

    runtime.set_wait_at_end(WaitAtEnd::No);
    Ok(())
}
