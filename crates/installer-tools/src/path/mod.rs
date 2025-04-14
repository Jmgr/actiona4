use std::{
    env::{join_paths, split_paths},
    path::Path,
};

use eyre::{Result, eyre};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix::{
    expand_environment_variables, normalize_path_entry, read_path_entries, strings_equal,
    write_path_entries,
};
#[cfg(windows)]
use windows::{
    expand_environment_variables, normalize_path_entry, read_path_entries, strings_equal,
    write_path_entries,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathScope {
    User,
    System,
}

pub fn add_directory_to_path(path_scope: PathScope, directory_path: &str) -> Result<bool> {
    let directory_entry = normalize_path_entry(directory_path);
    if directory_entry.is_empty() {
        return Err(eyre!("Directory path cannot be empty."));
    }

    let path_entries = read_path_entries(path_scope)?;
    if path_contains_entry(&path_entries, &directory_entry) {
        return Ok(false);
    }

    let mut updated_path_entries = path_entries;
    updated_path_entries.push(directory_entry);
    write_path_entries(path_scope, &updated_path_entries)?;
    Ok(true)
}

pub fn remove_directory_from_path(path_scope: PathScope, directory_path: &str) -> Result<bool> {
    let normalized_directory_path = normalize_path_entry(directory_path);
    if normalized_directory_path.is_empty() {
        return Err(eyre!("Directory path cannot be empty."));
    }

    let path_entries = read_path_entries(path_scope)?;
    let mut updated_path_entries = Vec::with_capacity(path_entries.len());
    let mut removed_any_entry = false;

    for path_entry in path_entries {
        if path_entry_matches(&path_entry, &normalized_directory_path) {
            removed_any_entry = true;
            continue;
        }

        updated_path_entries.push(path_entry);
    }

    if !removed_any_entry {
        return Ok(false);
    }

    write_path_entries(path_scope, &updated_path_entries)?;
    Ok(true)
}

pub fn is_directory_in_path(path_scope: PathScope, directory_path: &Path) -> Result<bool> {
    let directory_entry = normalize_path_entry(&directory_path.to_string_lossy());
    let path_entries = read_path_entries(path_scope)?;
    Ok(path_contains_entry(&path_entries, &directory_entry))
}

fn path_contains_entry(path_entries: &[impl AsRef<str>], directory_path: &str) -> bool {
    path_entries
        .iter()
        .any(|path_entry| path_entry_matches(path_entry.as_ref(), directory_path))
}

fn path_entry_matches(path_entry: &str, directory_path: &str) -> bool {
    let normalized_entry = normalize_path_entry(path_entry);
    if normalized_entry.is_empty() {
        return false;
    }

    let normalized_directory_path = normalize_path_entry(directory_path);
    if normalized_directory_path.is_empty() {
        return false;
    }

    if strings_equal(&normalized_entry, &normalized_directory_path) {
        return true;
    }

    let expanded_entry = normalize_path_entry(&expand_environment_variables(&normalized_entry));
    let expanded_directory_path =
        normalize_path_entry(&expand_environment_variables(&normalized_directory_path));

    strings_equal(&expanded_entry, &expanded_directory_path)
}

pub(super) fn strip_surrounding_quotes(path_entry: &str) -> &str {
    path_entry
        .strip_prefix('"')
        .and_then(|unquoted| unquoted.strip_suffix('"'))
        .unwrap_or(path_entry)
}

pub(super) fn split_path_entries(path_value: &str) -> Vec<String> {
    split_paths(path_value)
        .map(|path_entry| normalize_path_entry(&path_entry.to_string_lossy()))
        .filter(|path_entry| !path_entry.is_empty())
        .collect()
}

pub(super) fn join_path_entries(path_entries: &[impl AsRef<str>]) -> Result<String> {
    join_paths(
        path_entries
            .iter()
            .map(|path_entry| path_entry.as_ref())
            .filter(|path_entry| !path_entry.is_empty()),
    )
    .map(|path_value| path_value.to_string_lossy().into_owned())
    .map_err(|error| eyre!("Failed to serialize PATH entries: {error}"))
}
