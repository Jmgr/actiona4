use std::{env::current_exe, path::PathBuf, process::Stdio};

use color_eyre::{Result, eyre::eyre};
use tauri::{AppHandle, Manager};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    api::{
        point::point,
        rect::{Rect, rect},
        size::size,
    },
    cancel_on,
    error::CommonError,
};

#[instrument(skip_all)]
pub async fn ask_screenshot(
    app: &AppHandle,
    cancellation: CancellationToken,
) -> Result<Option<Rect>> {
    let selection_executable = resolve_selection_executable(app)?;
    let mut child = Command::new(&selection_executable);
    child
        .arg("rect")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let child = child.spawn().map_err(|error| {
        eyre!(
            "failed to launch selection executable {}: {error}",
            selection_executable.display()
        )
    })?;

    let output = match cancel_on(&cancellation, child.wait_with_output()).await {
        Ok(output) => output?,
        Err(error)
            if error
                .downcast_ref::<CommonError>()
                .is_some_and(CommonError::is_cancelled) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error.wrap_err("waiting for selection overlay failed")),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!(
            "selection overlay exited with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| eyre!("selection overlay stdout was not valid UTF-8: {error}"))?;

    if stdout.trim().is_empty() {
        return Ok(None);
    }

    parse_selection_rect(&stdout).map(Some)
}

fn resolve_selection_executable(app: &AppHandle) -> Result<PathBuf> {
    let executable_name = format!("selection{}", std::env::consts::EXE_SUFFIX);
    let mut candidates = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(&executable_name));
    }

    if let Ok(executable_path) = current_exe()
        && let Some(executable_dir) = executable_path.parent()
    {
        candidates.push(executable_dir.join(&executable_name));

        if let Some(parent_dir) = executable_dir.parent() {
            candidates.push(parent_dir.join(&executable_name));
        }
    }

    if let Some(path) = candidates.iter().find(|candidate| candidate.is_file()) {
        return Ok(path.clone());
    }

    Err(eyre!(
        "selection executable not found. Looked in: {}. \
         Bundled builds expect it in the Tauri resource directory, and development \
         builds can provide it via `cargo build -p selection`.",
        candidates
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn parse_selection_rect(stdout: &str) -> Result<Rect> {
    let mut last_error = None;

    for line in stdout
        .lines()
        .rev()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        match parse_selection_rect_line(line) {
            Ok(rect) => return Ok(rect),
            Err(error) => last_error = Some(error),
        }
    }

    if let Some(error) = last_error {
        return Err(error);
    }

    Err(eyre!("selection overlay returned no rectangle on stdout"))
}

fn parse_selection_rect_line(line: &str) -> Result<Rect> {
    let values = line
        .split_whitespace()
        .map(str::parse::<i32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| eyre!("invalid selection output `{line}`: {error}"))?;

    if values.len() != 4 {
        return Err(eyre!(
            "invalid selection output `{line}`: expected 4 integers"
        ));
    }

    let width = u32::try_from(values[2])
        .map_err(|error| eyre!("invalid selection width `{}`: {error}", values[2]))?;
    let height = u32::try_from(values[3])
        .map_err(|error| eyre!("invalid selection height `{}`: {error}", values[3]))?;

    if width == 0 || height == 0 {
        return Err(eyre!(
            "invalid selection output `{line}`: rectangle must have a non-zero size"
        ));
    }

    Ok(rect(point(values[0], values[1]), size(width, height)))
}

#[cfg(test)]
mod tests {
    use super::parse_selection_rect;
    use crate::api::{point::point, rect::rect, size::size};

    #[test]
    fn parses_selection_rect_from_last_line() {
        let stdout = "debug line\n10 20 30 40\n";

        assert_eq!(
            parse_selection_rect(stdout).unwrap(),
            rect(point(10, 20), size(30, 40))
        );
    }

    #[test]
    fn rejects_zero_sized_rectangles() {
        let error = parse_selection_rect("10 20 0 40\n").unwrap_err();

        assert!(error.to_string().contains("non-zero"));
    }
}
