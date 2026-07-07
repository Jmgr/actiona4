use macros::Parameter;
use types::platform::Platforms;

use crate::TranslationKey;

#[derive(Debug)]
pub struct EnumParameterVariant {
    pub id: &'static str,
    pub name: TranslationKey,
    pub platforms: Platforms,
}

// NOTE: storage is implemented by the ActionEnum derive macro.
#[derive(Debug, Parameter)]
pub struct EnumParameter {
    pub variants: &'static [EnumParameterVariant],
}

#[cfg(test)]
mod tests {
    use macros::ActionEnum;
    use serde::{Deserialize, Serialize};
    use types::platform::{PlatformConstraint, PlatformKind};

    use crate::parameters::{ParameterKind, ParameterStorage};

    #[derive(ActionEnum, Clone, Copy, Debug, Default, Deserialize, Serialize)]
    #[serde(rename_all = "kebab-case")]
    enum TestPlatformEnum {
        #[default]
        Everywhere,
        #[action_enum(only = Linux)]
        LinuxOnly,
        #[action_enum(not = [Wayland, X11])]
        NotLinuxDesktop,
    }

    #[test]
    fn variant_platform_constraints_are_captured() {
        let ParameterKind::Enum(enum_parameter) = TestPlatformEnum::KIND else {
            panic!("expected Enum parameter kind");
        };

        let everywhere = enum_parameter
            .variants
            .iter()
            .find(|variant| variant.id == "everywhere")
            .expect("everywhere variant");
        assert!(everywhere.platforms.is_unconstrained());

        let linux_only = enum_parameter
            .variants
            .iter()
            .find(|variant| variant.id == "linux-only")
            .expect("linux-only variant");
        assert_eq!(linux_only.platforms.0.len(), 1);
        assert!(matches!(
            linux_only.platforms.0[0],
            PlatformConstraint::Only(PlatformKind::Linux)
        ));

        let not_linux_desktop = enum_parameter
            .variants
            .iter()
            .find(|variant| variant.id == "not-linux-desktop")
            .expect("not-linux-desktop variant");
        assert_eq!(not_linux_desktop.platforms.0.len(), 2);
        assert!(matches!(
            not_linux_desktop.platforms.0[0],
            PlatformConstraint::Not(PlatformKind::Wayland)
        ));
        assert!(matches!(
            not_linux_desktop.platforms.0[1],
            PlatformConstraint::Not(PlatformKind::X11)
        ));
    }
}
