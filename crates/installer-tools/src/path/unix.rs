use std::io::ErrorKind;

use eyre::{Result, eyre};

use super::{PathScope, join_path_entries, split_path_entries, strip_surrounding_quotes};

const SYSTEM_ENVIRONMENT_FILE: &str = "/etc/environment";

pub(super) fn normalize_path_entry(path_entry: &str) -> String {
    let trimmed = strip_surrounding_quotes(path_entry.trim());
    if trimmed.is_empty() {
        return String::new();
    }

    let mut normalized = trimmed.to_string();
    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}

pub(super) fn strings_equal(left: &str, right: &str) -> bool {
    left == right
}

/// Expands `$VAR` and `${VAR}` references in a path entry.
pub(super) fn expand_environment_variables(path_entry: &str) -> String {
    let mut expanded = String::new();
    let mut remaining = path_entry;

    while let Some(dollar_index) = remaining.find('$') {
        expanded.push_str(&remaining[..dollar_index]);
        let after_dollar = &remaining[dollar_index + 1..];

        let (var_name, var_end_offset) = if after_dollar.starts_with('{') {
            // ${VAR} form
            if let Some(close_index) = after_dollar.find('}') {
                (&after_dollar[1..close_index], close_index + 1)
            } else {
                expanded.push('$');
                remaining = after_dollar;
                continue;
            }
        } else {
            // $VAR form: variable name is alphanumeric + underscores
            let end = after_dollar
                .find(|ch: char| !ch.is_alphanumeric() && ch != '_')
                .unwrap_or(after_dollar.len());
            (&after_dollar[..end], end)
        };

        if var_name.is_empty() {
            expanded.push('$');
            remaining = after_dollar;
            continue;
        }

        match std::env::var(var_name) {
            Ok(value) => expanded.push_str(&value),
            Err(_) => {
                expanded.push_str(&remaining[dollar_index..dollar_index + 1 + var_end_offset]);
            }
        }

        remaining = &after_dollar[var_end_offset..];
    }

    expanded.push_str(remaining);
    expanded
}

pub(super) fn read_path_entries(path_scope: PathScope) -> Result<Vec<String>> {
    match path_scope {
        PathScope::User => read_user_path_entries(),
        PathScope::System => read_system_path_entries(),
    }
}

pub(super) fn write_path_entries(
    path_scope: PathScope,
    path_entries: &[impl AsRef<str>],
) -> Result<()> {
    match path_scope {
        PathScope::User => write_user_path_entries(path_entries),
        PathScope::System => write_system_path_entries(path_entries),
    }
}

fn user_profile_path() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME").map_err(|_| eyre!("HOME environment variable not set"))?;
    Ok(std::path::PathBuf::from(home).join(".profile"))
}

/// Extracts the directory from a managed PATH export line in `~/.profile`.
///
/// Matches lines of the form:
/// `export PATH="${PATH}:/dir"` or `export PATH="$PATH:/dir"`
fn extract_profile_path_entry(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("export PATH=\"")?;
    let rest = rest
        .strip_prefix("${PATH}:")
        .or_else(|| rest.strip_prefix("$PATH:"))?;
    let dir = rest.strip_suffix('"')?;
    if dir.is_empty() {
        return None;
    }
    Some(dir.to_string())
}

fn read_user_path_entries() -> Result<Vec<String>> {
    let profile_path = user_profile_path()?;

    let content = match std::fs::read_to_string(&profile_path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => {
            return Err(eyre!("Failed to read {}: {error}", profile_path.display()));
        }
    };

    Ok(content
        .lines()
        .filter_map(extract_profile_path_entry)
        .collect())
}

fn write_user_path_entries(path_entries: &[impl AsRef<str>]) -> Result<()> {
    let profile_path = user_profile_path()?;

    let existing_content = match std::fs::read_to_string(&profile_path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(eyre!("Failed to read {}: {error}", profile_path.display()));
        }
    };

    // Keep all lines that are not our managed PATH exports
    let retained_lines: Vec<&str> = existing_content
        .lines()
        .filter(|line| extract_profile_path_entry(line).is_none())
        .collect();

    let mut new_content = retained_lines.join("\n");
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    for entry in path_entries {
        new_content.push_str(&format!("export PATH=\"${{PATH}}:{}\"\n", entry.as_ref()));
    }

    std::fs::write(&profile_path, new_content)
        .map_err(|error| eyre!("Failed to write {}: {error}", profile_path.display()))?;

    Ok(())
}

fn read_system_path_entries() -> Result<Vec<String>> {
    let content = match std::fs::read_to_string(SYSTEM_ENVIRONMENT_FILE) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => {
            return Err(eyre!("Failed to read {SYSTEM_ENVIRONMENT_FILE}: {error}"));
        }
    };

    for line in content.lines() {
        if let Some(path_value) = line.trim().strip_prefix("PATH=") {
            return Ok(split_path_entries(strip_surrounding_quotes(path_value)));
        }
    }

    Ok(vec![])
}

fn write_system_path_entries(path_entries: &[impl AsRef<str>]) -> Result<()> {
    let existing_content = match std::fs::read_to_string(SYSTEM_ENVIRONMENT_FILE) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(eyre!("Failed to read {SYSTEM_ENVIRONMENT_FILE}: {error}"));
        }
    };

    let retained_lines: Vec<&str> = existing_content
        .lines()
        .filter(|line| !line.trim().starts_with("PATH="))
        .collect();

    let path_value = join_path_entries(path_entries)?;

    let mut new_content = retained_lines.join("\n");
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    if !path_value.is_empty() {
        new_content.push_str(&format!("PATH=\"{path_value}\"\n"));
    }

    std::fs::write(SYSTEM_ENVIRONMENT_FILE, new_content)
        .map_err(|error| eyre!("Failed to write {SYSTEM_ENVIRONMENT_FILE}: {error}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        super::{join_path_entries, path_contains_entry, split_path_entries},
        expand_environment_variables, extract_profile_path_entry, normalize_path_entry,
    };

    #[test]
    fn normalize_path_entry_removes_trailing_slash() {
        assert_eq!(
            normalize_path_entry("/home/user/.local/bin/"),
            "/home/user/.local/bin"
        );
        assert_eq!(normalize_path_entry("/"), "/");
    }

    #[test]
    fn split_path_entries_splits_on_colon() {
        assert_eq!(
            split_path_entries("/usr/bin:/usr/local/bin"),
            vec![String::from("/usr/bin"), String::from("/usr/local/bin"),]
        );
    }

    #[test]
    fn join_path_entries_joins_with_colon() {
        assert_eq!(
            join_path_entries(&[String::from("/usr/bin"), String::from("/usr/local/bin"),])
                .unwrap(),
            "/usr/bin:/usr/local/bin"
        );
    }

    #[test]
    fn path_contains_entry_is_case_sensitive_on_unix() {
        assert!(path_contains_entry(
            &[String::from("/home/user/.local/bin")],
            "/home/user/.local/bin/"
        ));
        assert!(!path_contains_entry(
            &[String::from("/home/user/.local/bin")],
            "/Home/User/.local/bin"
        ));
    }

    #[test]
    fn expand_environment_variables_handles_dollar_brace_and_plain_forms() {
        unsafe { std::env::set_var("TEST_ACTIONA_VAR", "/expanded") };
        assert_eq!(
            expand_environment_variables("${TEST_ACTIONA_VAR}/bin"),
            "/expanded/bin"
        );
        assert_eq!(
            expand_environment_variables("$TEST_ACTIONA_VAR/bin"),
            "/expanded/bin"
        );
        unsafe { std::env::remove_var("TEST_ACTIONA_VAR") };
    }

    #[test]
    fn extract_profile_path_entry_parses_managed_lines() {
        assert_eq!(
            extract_profile_path_entry(r#"export PATH="${PATH}:/home/user/.local/bin""#),
            Some(String::from("/home/user/.local/bin"))
        );
        assert_eq!(
            extract_profile_path_entry(r#"export PATH="$PATH:/home/user/.local/bin""#),
            Some(String::from("/home/user/.local/bin"))
        );
        assert_eq!(extract_profile_path_entry("# just a comment"), None);
        assert_eq!(extract_profile_path_entry("export SOME_VAR=value"), None);
    }
}
