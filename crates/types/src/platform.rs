use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKind {
    Linux,
    Windows,
    X11,
    Wayland,
}

impl fmt::Display for PlatformKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::X11 => "X11",
            Self::Wayland => "Wayland",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformConstraint {
    Only(PlatformKind),
    Not(PlatformKind),
}

/// Static platform constraints for an action/parameter/enum variant.
///
/// An empty list means "works everywhere". `Only` entries form a union (any
/// match is enough), `Not` entries are vetoes (any match disqualifies).
#[derive(Clone, Copy, Debug, Default)]
pub struct Platforms(pub &'static [PlatformConstraint]);

impl Platforms {
    pub const ALL: Self = Self(&[]);

    pub const fn is_unconstrained(&self) -> bool {
        self.0.is_empty()
    }

    /// `active` describes the current session, e.g. an X11 session is
    /// `&[PlatformKind::Linux, PlatformKind::X11]`.
    pub fn is_supported(&self, active: &[PlatformKind]) -> bool {
        let mut has_only = false;
        let mut only_matched = false;

        for constraint in self.0 {
            match constraint {
                PlatformConstraint::Only(kind) => {
                    has_only = true;
                    if active.contains(kind) {
                        only_matched = true;
                    }
                }
                PlatformConstraint::Not(kind) => {
                    if active.contains(kind) {
                        return false;
                    }
                }
            }
        }

        !has_only || only_matched
    }
}

impl fmt::Display for Platforms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let only: Vec<String> = self
            .0
            .iter()
            .filter_map(|constraint| match constraint {
                PlatformConstraint::Only(kind) => Some(kind.to_string()),
                PlatformConstraint::Not(_) => None,
            })
            .collect();
        let not: Vec<String> = self
            .0
            .iter()
            .filter_map(|constraint| match constraint {
                PlatformConstraint::Not(kind) => Some(kind.to_string()),
                PlatformConstraint::Only(_) => None,
            })
            .collect();

        let mut parts = Vec::new();
        if !only.is_empty() {
            parts.push(format!("only works on {}", only.join(", ")));
        }
        if !not.is_empty() {
            parts.push(format!("does not work on {}", not.join(", ")));
        }

        write!(f, "{}", parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconstrained_is_supported_everywhere() {
        let platforms = Platforms::ALL;
        assert!(platforms.is_unconstrained());
        assert!(platforms.is_supported(&[PlatformKind::Windows]));
        assert!(platforms.is_supported(&[]));
    }

    #[test]
    fn only_forms_a_union() {
        let platforms = Platforms(&[
            PlatformConstraint::Only(PlatformKind::Linux),
            PlatformConstraint::Only(PlatformKind::Windows),
        ]);
        assert!(platforms.is_supported(&[PlatformKind::Linux, PlatformKind::X11]));
        assert!(platforms.is_supported(&[PlatformKind::Windows]));
        assert!(!platforms.is_supported(&[PlatformKind::Wayland]));
    }

    #[test]
    fn not_is_a_veto() {
        let platforms = Platforms(&[PlatformConstraint::Not(PlatformKind::Wayland)]);
        assert!(platforms.is_supported(&[PlatformKind::Linux, PlatformKind::X11]));
        assert!(!platforms.is_supported(&[PlatformKind::Linux, PlatformKind::Wayland]));
    }

    #[test]
    fn only_and_not_combine() {
        let platforms = Platforms(&[
            PlatformConstraint::Only(PlatformKind::Linux),
            PlatformConstraint::Not(PlatformKind::Wayland),
        ]);
        assert!(platforms.is_supported(&[PlatformKind::Linux, PlatformKind::X11]));
        assert!(!platforms.is_supported(&[PlatformKind::Linux, PlatformKind::Wayland]));
        assert!(!platforms.is_supported(&[PlatformKind::Windows]));
    }
}
