#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    X11,
    XWayland,
    Wayland,
    Windows,
}

impl Platform {
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

    #[must_use]
    pub const fn is_wayland(self) -> bool {
        matches!(self, Self::Wayland | Self::XWayland)
    }

    #[must_use]
    pub const fn is_linux(self) -> bool {
        !matches!(self, Self::Windows)
    }

    #[must_use]
    pub const fn is_windows(self) -> bool {
        matches!(self, Self::Windows)
    }
}
