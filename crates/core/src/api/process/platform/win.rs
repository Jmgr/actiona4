use color_eyre::Result;

use crate::{
    platform::win::process_info::{
        send_close_message_to_window, terminate_process_by_pid, windows_for_pid,
    },
    types::pid::Pid,
};

#[derive(Debug, Default)]
pub struct ProcessSignal {}

impl ProcessSignal {
    pub fn kill_by_pid(pid: Pid) -> Result<()> {
        let process_id: u32 = pid.into();
        terminate_process_by_pid(process_id)
    }

    pub fn terminate_by_pid(pid: Pid) -> Result<()> {
        let process_id: u32 = pid.into();
        let windows = windows_for_pid(process_id)?;
        let mut close_failed = false;

        for hwnd in windows.iter().copied() {
            if send_close_message_to_window(hwnd).is_err() {
                close_failed = true;
            }
        }

        if windows.is_empty() || close_failed {
            return terminate_process_by_pid(process_id);
        }

        Ok(())
    }
}
