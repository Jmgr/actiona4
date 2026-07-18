use action_definition::{
    actions::{
        ACTION_DEFINITIONS, ActionBranches, ActionDefinition, ActionInstance, flow::Or,
        random::RandomBranch,
    },
    parameters::ParameterKind,
    tree::BranchKind,
};

fn definition(id: &str) -> &'static ActionDefinition {
    ACTION_DEFINITIONS
        .iter()
        .find(|definition| definition.id == id)
        .unwrap_or_else(|| panic!("missing action definition {id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_supported_wait_actions_are_waitable() {
        for id in [
            "wait",
            "wait_until",
            "wait_while",
            "wait_for_button",
            "wait_for_clipboard_changed",
            "wait_for_movement",
            "wait_for_scroll",
        ] {
            assert!(definition(id).is_waitable, "{id} should be waitable");
        }

        for id in ["and", "or", "message_box", "move_cursor"] {
            assert!(!definition(id).is_waitable, "{id} should not be waitable");
        }
    }

    #[test]
    fn loop_actions_are_marked() {
        assert!(definition("for").is_looping);
        assert!(definition("for_each").is_looping);
        assert!(definition("loop").is_looping);
        assert!(definition("while").is_looping);
        assert!(!definition("wait").is_looping);
    }

    #[test]
    fn for_each_requires_an_array_parameter() {
        let array = definition("for_each")
            .parameters
            .iter()
            .find(|parameter| parameter.id == "array")
            .expect("for_each should have an array parameter");

        assert!(matches!(array.kind, ParameterKind::Array(_)));
    }

    #[test]
    fn collection_parameters_have_semantic_kinds() {
        let cases = definition("switch")
            .parameters
            .iter()
            .find(|parameter| parameter.id == "cases")
            .expect("switch should have a cases parameter");
        assert!(matches!(cases.kind, ParameterKind::LabelledBranches(_)));

        for id in ["code", "random_branch"] {
            let branches = definition(id)
                .parameters
                .iter()
                .find(|parameter| parameter.id == "branches")
                .unwrap_or_else(|| panic!("{id} should have a branches parameter"));
            assert!(matches!(branches.kind, ParameterKind::Branches(_)));
        }

        for id in ["and", "or"] {
            let inputs = definition(id)
                .parameters
                .iter()
                .find(|parameter| parameter.id == "inputs")
                .unwrap_or_else(|| panic!("{id} should have an inputs parameter"));
            assert!(matches!(inputs.kind, ParameterKind::ActionList(_)));
        }
    }

    #[test]
    fn random_has_one_named_branch_per_configured_branch() {
        let action = RandomBranch {
            branches: vec!["first".to_owned(), "second".to_owned()].into(),
        };

        assert_eq!(
            action.action_branches(),
            vec![
                BranchKind::Named("first".to_owned()),
                BranchKind::Named("second".to_owned()),
            ]
        );
    }

    #[test]
    fn or_has_one_positional_branch_per_input() {
        let action = Or {
            inputs: vec![
                (definition("wait").create_instance)(),
                (definition("wait_for_button").create_instance)(),
            ]
            .into(),
        };

        assert_eq!(
            action.action_branches(),
            vec![
                BranchKind::Named("0".to_owned()),
                BranchKind::Named("1".to_owned()),
            ]
        );

        let instance = ActionInstance::Or(action.into());
        assert!(matches!(instance, ActionInstance::Or(_)));
    }
}
