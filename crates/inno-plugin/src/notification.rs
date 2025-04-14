use std::ffi::{c_char, c_int};

use eyre::Result;
use installer_tools::notification::{
    register_notification_application, unregister_notification_application,
};

use crate::ffi::{export_result_to_error_buffer, parse_required_string};

#[unsafe(no_mangle)]
pub extern "C" fn register_notification_app(
    aumid: *const c_char,
    display_name: *const c_char,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_result_to_error_buffer(
        run_notification_operation(
            aumid,
            Some(display_name),
            |parsed_aumid, parsed_display_name| {
                register_notification_application(
                    parsed_aumid,
                    parsed_display_name.expect("display name should be present"),
                )
            },
        ),
        error_string,
        error_string_capacity,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn unregister_notification_app(
    aumid: *const c_char,
    error_string: *mut c_char,
    error_string_capacity: usize,
) -> c_int {
    export_result_to_error_buffer(
        run_notification_operation(aumid, None, |parsed_aumid, _parsed_display_name| {
            unregister_notification_application(parsed_aumid)
        }),
        error_string,
        error_string_capacity,
    )
}

fn run_notification_operation(
    aumid: *const c_char,
    display_name: Option<*const c_char>,
    operation: impl FnOnce(&str, Option<&str>) -> Result<()>,
) -> Result<()> {
    let parsed_aumid = parse_required_string(aumid, "AUMID")?;
    let parsed_display_name = match display_name {
        Some(display_name) => Some(parse_required_string(display_name, "Display name")?),
        None => None,
    };

    operation(&parsed_aumid, parsed_display_name.as_deref())
}
