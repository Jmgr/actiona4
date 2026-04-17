/// Skip the test at runtime when a condition is not met.
#[macro_export]
macro_rules! skip_unless {
    ($cond:expr, $reason:literal) => {
        if !($cond) {
            println!("skipping: {}", $reason);
            return;
        }
    };
}

/// Skip unless running on Windows (compile-time).
#[macro_export]
macro_rules! require_windows {
    () => {
        #[cfg(not(windows))]
        {
            println!("skipping: requires Windows");
            return;
        }
    };
}

/// Skip when running on Windows (compile-time).
#[macro_export]
macro_rules! require_not_windows {
    () => {
        #[cfg(windows)]
        {
            println!("skipping: not supported on Windows");
            return;
        }
    };
}

/// Skip on pure Wayland; allow X11 and XWayland (runtime check).
#[macro_export]
macro_rules! require_not_wayland {
    () => {{
        let session = std::env::var("XDG_SESSION_TYPE").ok();
        let is_pure_wayland = session.as_deref() == Some("wayland")
            || (session.as_deref() != Some("x11")
                && std::env::var_os("WAYLAND_DISPLAY").is_some()
                && std::env::var_os("DISPLAY").is_none());
        if is_pure_wayland {
            println!("skipping: not supported on pure Wayland");
            return;
        }
    }};
}

/// Return the path to the actiona-run binary built by cargo.
///
/// Walks up from the running test binary (target/debug/deps/<exe>) to
/// target/debug/ and appends the binary name.
pub fn actiona_run_bin() -> std::path::PathBuf {
    // current_exe → target/{profile}/deps/<test-exe>
    // parent()    → target/{profile}/deps/
    // parent()    → target/{profile}/
    let mut path = std::env::current_exe()
        .expect("cannot determine test binary path")
        .parent()
        .expect("no parent")
        .parent()
        .expect("no grandparent")
        .to_path_buf();

    path.push(if cfg!(windows) {
        "actiona-run.exe"
    } else {
        "actiona-run"
    });

    path
}
