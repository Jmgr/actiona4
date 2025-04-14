use std::ffi::{c_char, c_int};

use eyre::{Result, eyre};
use installer_tools::path::{
    PathScope, add_directory_to_path as add_path_entry,
    remove_directory_from_path as remove_path_entry,
};

use crate::ffi::{export_result_to_error_buffer, parse_required_string};

#[unsafe(no_mangle)]
pub extern "C" fn add_directory_to_path(
    path_scope: c_int,
    directory_path: *const c_char,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_result_to_error_buffer(
        run_path_operation(path_scope, directory_path, |scope, path| {
            add_path_entry(scope, path).map(|_| ())
        }),
        error_string,
        error_string_capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn remove_directory_from_path(
    path_scope: c_int,
    directory_path: *const c_char,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_result_to_error_buffer(
        run_path_operation(path_scope, directory_path, |scope, path| {
            remove_path_entry(scope, path).map(|_| ())
        }),
        error_string,
        error_string_capacity,
    )
}

fn run_path_operation(
    path_scope: c_int,
    directory_path: *const c_char,
    operation: impl FnOnce(PathScope, &str) -> Result<()>,
) -> Result<()> {
    let parsed_scope = parse_path_scope(path_scope)?;
    let directory_path = parse_required_string(directory_path, "Directory path")?;
    operation(parsed_scope, &directory_path)
}

fn parse_path_scope(path_scope: c_int) -> Result<PathScope> {
    match path_scope {
        0 => Ok(PathScope::User),
        1 => Ok(PathScope::System),
        _ => Err(eyre!("Invalid PATH scope value: {path_scope}.")),
    }
}
