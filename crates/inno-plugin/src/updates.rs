use std::ffi::{c_char, c_int};

use installer_tools::updates::check_for_update as check_for_update_impl;

use crate::ffi::{clear_string_buffer, write_string_to_buffer};

const UPDATE_CHECK_NO_UPDATE: c_int = 0;
const UPDATE_CHECK_AVAILABLE: c_int = 1;
const UPDATE_CHECK_ERROR: c_int = 2;

#[unsafe(no_mangle)]
pub extern "C" fn check_for_update(
    version_string: *mut c_char,
    version_string_capacity: usize,
    download_url: *mut c_char,
    download_url_capacity: usize,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    match check_for_update_impl() {
        Ok(Some(update_check_result)) => {
            write_string_to_buffer(
                &update_check_result.version,
                version_string,
                version_string_capacity,
            );
            write_string_to_buffer(
                &update_check_result.download_url,
                download_url,
                download_url_capacity,
            );
            clear_string_buffer(error_string, error_string_capacity);
            UPDATE_CHECK_AVAILABLE
        }
        Ok(None) => {
            clear_string_buffer(version_string, version_string_capacity);
            clear_string_buffer(download_url, download_url_capacity);
            clear_string_buffer(error_string, error_string_capacity);
            UPDATE_CHECK_NO_UPDATE
        }
        Err(error) => {
            clear_string_buffer(version_string, version_string_capacity);
            clear_string_buffer(download_url, download_url_capacity);
            write_string_to_buffer(&error.to_string(), error_string, error_string_capacity);
            UPDATE_CHECK_ERROR
        }
    }
}
