use action_definition::actions::{
    ActionInstance, click::Click, code::Code, message_box::MessageBox, test::Test,
};
use serde_json::json;

/// Serialize → deserialize → serialize and assert the wire format is stable.
fn assert_roundtrips(instance: ActionInstance) {
    let json = serde_json::to_value(&instance).expect("serialize");
    let back: ActionInstance =
        serde_json::from_value(json.clone()).expect("deserialize what we serialized");
    let json_again = serde_json::to_value(&back).expect("re-serialize");
    assert_eq!(json, json_again, "round-trip changed the wire format");
}

#[test]
fn click_roundtrips() {
    assert_roundtrips(ActionInstance::Click(Click::default()));
}

#[test]
fn message_box_roundtrips() {
    assert_roundtrips(ActionInstance::MessageBox(MessageBox::default()));
}

#[test]
fn code_roundtrips() {
    assert_roundtrips(ActionInstance::Code(Code::default()));
}

#[test]
fn test_roundtrips() {
    assert_roundtrips(ActionInstance::Test(Test::default()));
}

#[test]
fn message_box_wire_format() {
    let json =
        serde_json::to_value(ActionInstance::MessageBox(MessageBox::default())).expect("serialize");

    assert_eq!(
        json,
        json!({
            "kind": "message_box",
            "title": { "mode": "static", "value": "" },
            "text": { "mode": "static", "value": "" },
            "buttons": "ok",
        })
    );
}

#[test]
fn test_wire_format() {
    let json = serde_json::to_value(ActionInstance::Test(Test::default())).expect("serialize");

    assert_eq!(
        json,
        json!({
            "kind": "test",
            "percent": { "mode": "static", "value": 0 },
        })
    );
}

#[test]
fn code_wire_format() {
    let json = serde_json::to_value(ActionInstance::Code(Code::default())).expect("serialize");

    assert_eq!(
        json,
        json!({
            "kind": "code",
            "source": "",
        })
    );
}
