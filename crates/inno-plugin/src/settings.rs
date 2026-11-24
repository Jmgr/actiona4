use std::ffi::{c_char, c_int};

use installer_tools::settings::{
    get_telemetry_enabled as get_telemetry_enabled_impl,
    get_update_check_enabled as get_update_check_enabled_impl,
    set_telemetry_enabled as set_telemetry_enabled_impl,
    set_update_check_enabled as set_update_check_enabled_impl,
};

use crate::ffi::{export_bool_result, export_result_to_error_buffer, parse_bool};

#[unsafe(no_mangle)]
pub extern "C" fn get_update_check_enabled(
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_bool_result(
        get_update_check_enabled_impl(),
        error_string,
        error_string_capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn set_update_check_enabled(
    update_check_enabled: c_int,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_result_to_error_buffer(
        run_set_update_check_enabled(update_check_enabled),
        error_string,
        error_string_capacity,
    )
}

fn run_set_update_check_enabled(update_check_enabled: c_int) -> eyre::Result<()> {
    let update_check_enabled = parse_bool(update_check_enabled, "Update check enabled")?;
    set_update_check_enabled_impl(update_check_enabled)
}

#[unsafe(no_mangle)]
pub extern "C" fn get_telemetry_enabled(
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_bool_result(
        get_telemetry_enabled_impl(),
        error_string,
        error_string_capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn set_telemetry_enabled(
    telemetry_enabled: c_int,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_result_to_error_buffer(
        run_set_telemetry_enabled(telemetry_enabled),
        error_string,
        error_string_capacity,
    )
}

fn run_set_telemetry_enabled(telemetry_enabled: c_int) -> eyre::Result<()> {
    let telemetry_enabled = parse_bool(telemetry_enabled, "Telemetry enabled")?;
    set_telemetry_enabled_impl(telemetry_enabled)
}
