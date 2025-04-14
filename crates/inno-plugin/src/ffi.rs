use std::{
    ffi::{CStr, CString, c_char, c_int},
    ptr,
};

use eyre::{Result, eyre};

pub fn parse_required_string(value: *const c_char, value_name: &str) -> Result<String> {
    if value.is_null() {
        return Err(eyre!("{value_name} pointer cannot be null."));
    }

    #[allow(unsafe_code)]
    let value = unsafe { CStr::from_ptr(value) };
    let value = value.to_str().map_err(|error| eyre!(error))?.trim();
    if value.is_empty() {
        return Err(eyre!("{value_name} cannot be empty."));
    }

    Ok(value.to_owned())
}

pub fn export_result_to_error_buffer(
    result: Result<()>,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    match result {
        Ok(()) => {
            clear_string_buffer(error_string, error_string_capacity);
            0
        }
        Err(error) => {
            write_error_message(&error.to_string(), error_string, error_string_capacity);
            1
        }
    }
}

pub fn parse_bool(value: c_int, value_name: &str) -> Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(eyre!("{value_name} must be 0 or 1.")),
    }
}

pub fn clear_string_buffer(string_buffer: *mut c_char, string_buffer_capacity: usize) {
    if string_buffer.is_null() || string_buffer_capacity == 0 {
        return;
    }

    #[allow(unsafe_code)]
    unsafe {
        *string_buffer = 0;
    }
}

pub fn write_string_to_buffer(
    value: &str,
    string_buffer: *mut c_char,
    string_buffer_capacity: usize,
) {
    write_c_string_to_buffer(
        value,
        "Failed to serialize string.",
        string_buffer,
        string_buffer_capacity,
    );
}

fn write_c_string_to_buffer(
    value: &str,
    fallback_message: &str,
    string_buffer: *mut c_char,
    string_buffer_capacity: usize,
) {
    if string_buffer.is_null() || string_buffer_capacity == 0 {
        return;
    }

    let c_string = match CString::new(value) {
        Ok(value) => value,
        Err(_) => match CString::new(fallback_message) {
            Ok(fallback_message) => fallback_message,
            Err(_) => return,
        },
    };
    let bytes = c_string.as_bytes_with_nul();
    let copy_len = bytes.len().min(string_buffer_capacity);

    #[allow(unsafe_code)]
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), string_buffer, copy_len);
        if copy_len == string_buffer_capacity {
            *string_buffer.add(string_buffer_capacity - 1) = 0;
        }
    }
}

fn write_error_message(message: &str, error_string: *mut c_char, error_string_capacity: usize) {
    write_c_string_to_buffer(
        message,
        "Failed to serialize error message.",
        error_string,
        error_string_capacity,
    );
}
