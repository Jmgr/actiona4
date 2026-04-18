use strum::{Display, EnumIs};

#[derive(Clone, Copy, Debug, Display, EnumIs, Eq, PartialEq)]
pub enum Platform {
    #[strum(to_string = "x11")]
    X11,
    #[strum(to_string = "xwayland")]
    XWayland,
    #[strum(to_string = "wayland")]
    Wayland,
    #[strum(to_string = "windows")]
    Windows,
}

impl Platform {
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn detect() -> Self {
        #[cfg(windows)]
        return Self::Windows;

        #[cfg(unix)]
        {
            use std::env;
            match env::var("XDG_SESSION_TYPE").ok().as_deref() {
                Some("wayland") => Self::Wayland,
                Some("x11") => Self::X11,
                _ => {
                    if env::var_os("WAYLAND_DISPLAY").is_some() {
                        Self::XWayland
                    } else {
                        Self::X11
                    }
                }
            }
        }
    }
}

#[must_use]
#[cfg(linux)]
pub const fn is_linux() -> bool {
    true
}

#[cfg(not(linux))]
pub const fn is_linux() -> bool {
    false
}

#[must_use]
#[cfg(unix)]
pub const fn is_unix() -> bool {
    true
}

#[cfg(not(unix))]
pub const fn is_unix() -> bool {
    false
}

#[must_use]
#[cfg(windows)]
pub const fn is_windows() -> bool {
    true
}

#[must_use]
#[cfg(not(windows))]
pub const fn is_windows() -> bool {
    false
}
