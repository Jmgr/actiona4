#[cfg(test)]
mod tests {
    use action_definition::actions::{clipboard::get_text::GetClipboardText, mouse::click::Click};
    use types::platform::{PlatformConstraint, PlatformKind};

    #[test]
    fn unconstrained_action_and_parameters_are_supported_everywhere() {
        assert!(Click::DEFINITION.platforms.is_unconstrained());
        for parameter in Click::DEFINITION.parameters {
            assert!(
                parameter.platforms.is_unconstrained(),
                "parameter {} should be unconstrained",
                parameter.id
            );
        }
    }

    #[test]
    fn linux_only_parameter_is_flagged() {
        let selection = GetClipboardText::DEFINITION
            .parameters
            .iter()
            .find(|parameter| parameter.id == "selection")
            .expect("selection parameter");

        assert_eq!(selection.platforms.0.len(), 1);
        assert!(matches!(
            selection.platforms.0[0],
            PlatformConstraint::Only(PlatformKind::Linux)
        ));

        assert!(
            selection
                .platforms
                .is_supported(&[PlatformKind::Linux, PlatformKind::X11])
        );
        assert!(!selection.platforms.is_supported(&[PlatformKind::Windows]));
    }
}
