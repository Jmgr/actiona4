use std::path::Path;

use color_eyre::{Result, eyre::Context};
use tokio::process::{Child, Command};
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;

use super::{RunningScriptBase, ScriptPlatform};

pub type RunningScript = RunningScriptBase<WindowsScript>;

pub struct WindowsScript;

impl ScriptPlatform for WindowsScript {
    fn new(_child: &mut Child) -> Result<Self> {
        Ok(Self)
    }

    fn configure_command(command: &mut Command) {
        // Only the supervisor receives terminal Ctrl+C; the child stops through its socket.
        command.creation_flags(CREATE_NEW_PROCESS_GROUP.0);
    }

    fn kill_child(child: &mut Child) -> Result<()> {
        child.start_kill().context("terminating watched script")
    }
}

#[allow(unsafe_code)]
pub fn paths_match(left: &Path, right: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt as _;

    use windows::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};

    // Windows opens paths without regard to casing, but notifications use the on-disk spelling.
    let left: Vec<_> = left.as_os_str().encode_wide().collect();
    let right: Vec<_> = right.as_os_str().encode_wide().collect();
    // SAFETY: both slices contain valid UTF-16 buffers with explicit lengths.
    unsafe { CompareStringOrdinal(&left, &right, true) == CSTR_EQUAL }
}
