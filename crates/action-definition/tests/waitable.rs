use action_definition::{
    actions::{ACTION_DEFINITIONS, ActionBranches, ActionInstance, flow::Or},
    parameters::ParameterKind,
    tree::BranchKind,
};

fn definition(id: &str) -> &'static action_definition::actions::ActionDefinition {
    ACTION_DEFINITIONS
        .iter()
        .find(|definition| definition.id == id)
        .unwrap_or_else(|| panic!("missing action definition {id}"))
}

#[test]
fn only_supported_wait_actions_are_waitable() {
    for id in [
        "wait",
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
fn or_has_one_positional_branch_per_input() {
    let action = Or {
        inputs: vec![
            (definition("wait").create_instance)(),
            (definition("wait_for_button").create_instance)(),
        ],
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
