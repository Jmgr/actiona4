use std::{env::current_exe, path::PathBuf, process::Stdio};

use color_eyre::{Result, eyre::eyre};
use serde::Deserialize;
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

#[derive(Deserialize)]
struct RectSelection {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[instrument(skip_all)]
pub async fn ask_screenshot(cancellation: CancellationToken) -> Result<Option<Rect>> {
    let selection_executable = resolve_selection_executable()?;
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

fn resolve_selection_executable() -> Result<PathBuf> {
    let executable_name = format!("selection-tool{}", std::env::consts::EXE_SUFFIX);
    let mut candidates = Vec::new();

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
         Build it with `cargo build -p selection`.",
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
    let selection: RectSelection = serde_json::from_str(line)
        .map_err(|error| eyre!("invalid selection output `{line}`: {error}"))?;

    let width = u32::try_from(selection.width)
        .map_err(|error| eyre!("invalid selection width `{}`: {error}", selection.width))?;
    let height = u32::try_from(selection.height)
        .map_err(|error| eyre!("invalid selection height `{}`: {error}", selection.height))?;

    if width == 0 || height == 0 {
        return Err(eyre!(
            "invalid selection output `{line}`: rectangle must have a non-zero size"
        ));
    }

    Ok(rect(point(selection.x, selection.y), size(width, height)))
}

#[cfg(test)]
mod tests {
    use super::parse_selection_rect;
    use crate::api::{point::point, rect::rect, size::size};

    #[test]
    fn parses_selection_rect_from_last_line() {
        let stdout = "debug line\n{\"x\":10,\"y\":20,\"width\":30,\"height\":40}\n";

        assert_eq!(
            parse_selection_rect(stdout).unwrap(),
            rect(point(10, 20), size(30, 40))
        );
    }

    #[test]
    fn rejects_zero_sized_rectangles() {
        let error =
            parse_selection_rect("{\"x\":10,\"y\":20,\"width\":0,\"height\":40}\n").unwrap_err();

        assert!(error.to_string().contains("non-zero"));
    }
}
