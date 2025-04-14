use std::io::ErrorKind;

use eyre::{Result, eyre};
use winreg::{
    RegKey, RegValue,
    enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ,
        REG_SZ,
    },
};

use super::{PathScope, join_path_entries, split_path_entries, strip_surrounding_quotes};

impl PathScope {
    pub(super) const fn registry_subkey(self) -> &'static str {
        match self {
            Self::User => "Environment",
            Self::System => r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        }
    }
}

pub(super) fn normalize_path_entry(path_entry: &str) -> String {
    let trimmed_path_entry = strip_surrounding_quotes(path_entry.trim());
    if trimmed_path_entry.is_empty() {
        return String::new();
    }

    let mut normalized_path_entry = trimmed_path_entry.replace('/', "\\");

    while normalized_path_entry.ends_with('\\') && !is_windows_root(&normalized_path_entry) {
        normalized_path_entry.pop();
    }

    normalized_path_entry
}

fn is_windows_root(path_entry: &str) -> bool {
    if path_entry == r"\" {
        return true;
    }

    let path_bytes = path_entry.as_bytes();
    path_bytes.len() == 3
        && path_bytes[1] == b':'
        && path_bytes[2] == b'\\'
        && path_bytes[0].is_ascii_alphabetic()
}

pub(super) fn strings_equal(left: &str, right: &str) -> bool {
    left.to_uppercase() == right.to_uppercase()
}

pub(super) fn expand_environment_variables(path_entry: &str) -> String {
    let mut expanded_path_entry = String::new();
    let mut remaining_path = path_entry;

    while let Some(prefix_index) = remaining_path.find('%') {
        expanded_path_entry.push_str(&remaining_path[..prefix_index]);
        let variable_start_index = prefix_index + 1;

        if let Some(suffix_offset) = remaining_path[variable_start_index..].find('%') {
            let variable_end_index = variable_start_index + suffix_offset;
            let variable_name = &remaining_path[variable_start_index..variable_end_index];

            if variable_name.is_empty() {
                expanded_path_entry.push('%');
                remaining_path = &remaining_path[variable_start_index..];
                continue;
            }

            match std::env::var(variable_name) {
                Ok(variable_value) => expanded_path_entry.push_str(&variable_value),
                Err(_) => {
                    expanded_path_entry
                        .push_str(&remaining_path[prefix_index..=variable_end_index]);
                }
            }

            remaining_path = &remaining_path[variable_end_index + 1..];
            continue;
        }

        expanded_path_entry.push_str(&remaining_path[prefix_index..]);
        return expanded_path_entry;
    }

    expanded_path_entry.push_str(remaining_path);
    expanded_path_entry
}

pub(super) fn read_path_entries(path_scope: PathScope) -> Result<Vec<String>> {
    read_path_value(path_scope)
        .map(|path_value| split_path_entries(&path_value.unwrap_or_default()))
}

pub(super) fn write_path_entries(
    path_scope: PathScope,
    path_entries: &[impl AsRef<str>],
) -> Result<()> {
    let path_value = join_path_entries(path_entries)?;
    write_path_value(
        path_scope,
        if path_value.is_empty() {
            None
        } else {
            Some(&path_value)
        },
    )
}

fn read_path_value(path_scope: PathScope) -> Result<Option<String>> {
    let registry_key = match registry_root_key(path_scope)
        .open_subkey_with_flags(path_scope.registry_subkey(), KEY_QUERY_VALUE)
    {
        Ok(registry_key) => registry_key,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(eyre!(
                "Failed to open {} PATH registry key: {error}",
                path_scope_name(path_scope)
            ));
        }
    };

    let path_value = match registry_key.get_raw_value("Path") {
        Ok(path_value) => path_value,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(eyre!(
                "Failed to read {} PATH value: {error}",
                path_scope_name(path_scope)
            ));
        }
    };

    if path_value.vtype != REG_SZ && path_value.vtype != REG_EXPAND_SZ {
        return Err(eyre!(
            "{} PATH registry value has unsupported type {}.",
            path_scope_name(path_scope),
            path_value.vtype as u32
        ));
    }

    decode_registry_string(&path_value.bytes).map(Some)
}

fn write_path_value(path_scope: PathScope, path_value: Option<&str>) -> Result<()> {
    let (registry_key, _) = registry_root_key(path_scope)
        .create_subkey_with_flags(path_scope.registry_subkey(), KEY_SET_VALUE)
        .map_err(|error| {
            eyre!(
                "Failed to open {} PATH registry key for writing: {error}",
                path_scope_name(path_scope)
            )
        })?;

    match path_value {
        Some(path_value) => registry_key
            .set_raw_value("Path", &encode_expand_string(path_value))
            .map_err(|error| {
                eyre!(
                    "Failed to write {} PATH value: {error}",
                    path_scope_name(path_scope)
                )
            })?,
        None => match registry_key.delete_value("Path") {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(eyre!(
                    "Failed to delete empty {} PATH value: {error}",
                    path_scope_name(path_scope)
                ));
            }
        },
    }

    Ok(())
}

fn registry_root_key(path_scope: PathScope) -> RegKey {
    match path_scope {
        PathScope::User => RegKey::predef(HKEY_CURRENT_USER),
        PathScope::System => RegKey::predef(HKEY_LOCAL_MACHINE),
    }
}

fn encode_expand_string(path_value: &str) -> RegValue<'_> {
    let mut bytes = Vec::with_capacity((path_value.encode_utf16().count() + 1) * 2);
    for path_value_unit in path_value.encode_utf16().chain(std::iter::once(0)) {
        bytes.extend_from_slice(&path_value_unit.to_le_bytes());
    }

    RegValue {
        bytes: bytes.into(),
        vtype: REG_EXPAND_SZ,
    }
}

fn decode_registry_string(path_value_bytes: &[u8]) -> Result<String> {
    if !path_value_bytes.len().is_multiple_of(2) {
        return Err(eyre!("Registry string value has an odd byte length."));
    }

    let mut path_value_units = Vec::with_capacity(path_value_bytes.len() / 2);
    for unit_index in (0..path_value_bytes.len()).step_by(2) {
        path_value_units.push(u16::from_le_bytes([
            path_value_bytes[unit_index],
            path_value_bytes[unit_index + 1],
        ]));
    }

    while path_value_units.last().copied() == Some(0) {
        path_value_units.pop();
    }

    String::from_utf16(&path_value_units).map_err(|error| eyre!(error))
}

fn path_scope_name(path_scope: PathScope) -> &'static str {
    match path_scope {
        PathScope::User => "user",
        PathScope::System => "system",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{join_path_entries, path_contains_entry, split_path_entries},
        normalize_path_entry,
    };

    #[test]
    fn normalize_path_entry_removes_wrapping_quotes_and_trailing_backslash() {
        assert_eq!(
            normalize_path_entry(r#""C:\Actiona Run\""#),
            r"C:\Actiona Run"
        );
        assert_eq!(normalize_path_entry(r"C:\"), r"C:\");
    }

    #[test]
    fn split_path_entries_keeps_quoted_semicolon_entries_together() {
        assert_eq!(
            split_path_entries(r#"C:\one;"C:\two;three";C:\four"#),
            vec![
                String::from(r"C:\one"),
                String::from(r"C:\two;three"),
                String::from(r"C:\four"),
            ]
        );
    }

    #[test]
    fn join_path_entries_quotes_entries_with_semicolons() {
        assert_eq!(
            join_path_entries(&[String::from(r"C:\one"), String::from(r"C:\two;three"),]).unwrap(),
            r#"C:\one;"C:\two;three""#
        );
    }

    #[test]
    fn path_contains_entry_is_case_insensitive_and_ignores_trailing_backslash() {
        assert!(path_contains_entry(
            &[String::from(r"C:\Actiona Run")],
            "c:\\actiona run\\"
        ));
    }
}
