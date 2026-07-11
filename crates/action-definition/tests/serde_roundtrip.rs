use std::time::Duration;

use action_definition::{
    actions::{
        ActionInstance,
        flow::{And, Break, Continue, Or},
        misc::test::Test,
        mouse::click::Click,
        system::code::Code,
        window::message_box::MessageBox,
    },
    parameters::duration::DurationValue,
    scriptable::Scriptable,
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
    assert_roundtrips(ActionInstance::Click(Click::default().into()));
}

#[test]
fn action_parameters_expose_runtime_names() {
    assert_eq!(Click::default().position.name(), "position");
    assert_eq!(MessageBox::default().buttons.name(), "buttons");
}

#[test]
fn message_box_roundtrips() {
    assert_roundtrips(ActionInstance::MessageBox(MessageBox::default().into()));
}

#[test]
fn code_roundtrips() {
    assert_roundtrips(ActionInstance::Code(Code::default().into()));
}

#[test]
fn test_roundtrips() {
    assert_roundtrips(ActionInstance::Test(Test::default().into()));
}

#[test]
fn and_roundtrips() {
    assert_roundtrips(ActionInstance::And(And::default().into()));
}

#[test]
fn or_roundtrips() {
    assert_roundtrips(ActionInstance::Or(Or::default().into()));
}

#[test]
fn break_roundtrips() {
    assert_roundtrips(ActionInstance::Break(Break::default().into()));
}

#[test]
fn continue_roundtrips() {
    assert_roundtrips(ActionInstance::Continue(Continue::default().into()));
}

#[test]
fn message_box_wire_format() {
    let json = serde_json::to_value(ActionInstance::MessageBox(MessageBox::default().into()))
        .expect("serialize");

    assert_eq!(
        json,
        json!({
            "kind": "message_box",
            "title": { "mode": "static", "value": null },
            "text": { "mode": "static", "value": "" },
            "buttons": "ok",
            "icon": { "mode": "static", "value": null },
            "ok_label": { "mode": "static", "value": null },
            "yes_label": { "mode": "static", "value": null },
            "no_label": { "mode": "static", "value": null },
            "cancel_label": { "mode": "static", "value": null },
        })
    );
}

#[test]
fn test_wire_format() {
    let json =
        serde_json::to_value(ActionInstance::Test(Test::default().into())).expect("serialize");

    assert_eq!(
        json,
        json!({
            "kind": "test",
            "percent": { "mode": "static", "value": 50 },
            "duration": { "mode": "static", "value": "0s" },
        })
    );
}

#[test]
fn code_wire_format() {
    let json =
        serde_json::to_value(ActionInstance::Code(Code::default().into())).expect("serialize");

    assert_eq!(
        json,
        json!({
            "kind": "code",
            "source": "",
        })
    );
}

#[test]
fn duration_value_wire_format() {
    let duration = DurationValue::new(Duration::new(1, 500_000_000));
    let json = serde_json::to_value(duration).expect("serialize duration");

    assert_eq!(json, json!("1s 500ms"));

    let back: DurationValue = serde_json::from_value(json).expect("deserialize duration");
    assert_eq!(back, duration);
}

#[test]
fn duration_value_accepts_legacy_duration_object() {
    let duration: DurationValue = serde_json::from_value(json!({
        "secs": 1,
        "nanos": 500_000_000,
    }))
    .expect("deserialize legacy duration object");

    assert_eq!(duration, DurationValue::new(Duration::new(1, 500_000_000)));
}

#[test]
fn scriptable_optional_duration_wire_format() {
    let duration: Scriptable<Option<DurationValue>> =
        Scriptable::new_static(Some(DurationValue::new(Duration::from_millis(250))));
    let json = serde_json::to_value(duration).expect("serialize duration parameter");

    assert_eq!(
        json,
        json!({
            "mode": "static",
            "value": "250ms",
        })
    );

    let back: Scriptable<Option<DurationValue>> =
        serde_json::from_value(json).expect("deserialize duration parameter");
    assert!(matches!(
        back,
        Scriptable::Static {
            value: Some(value)
        } if value == DurationValue::new(Duration::from_millis(250))
    ));
}

#[test]
fn duration_parameter_accepts_duration_like_string_value() {
    let action: ActionInstance = serde_json::from_value(json!({
        "kind": "test",
        "percent": { "mode": "static", "value": 0 },
        "duration": { "mode": "static", "value": "1.5s" },
    }))
    .expect("deserialize test action with duration-like value");

    let ActionInstance::Test(test) = action else {
        panic!("expected test action");
    };
    assert!(matches!(
        test.duration.value(),
        Scriptable::Static { value } if *value == DurationValue::new(Duration::from_millis(1_500))
    ));
}

#[test]
fn optional_duration_parameter_accepts_duration_like_number_value() {
    let action: ActionInstance = serde_json::from_value(json!({
        "kind": "click",
        "position": { "mode": "static", "value": null },
        "button": { "mode": "static", "value": "left" },
        "relative_position": { "mode": "static", "value": false },
        "amount": { "mode": "static", "value": null },
        "interval": { "mode": "static", "value": 250 },
        "duration": { "mode": "static", "value": null },
    }))
    .expect("deserialize click action with duration-like value");

    let ActionInstance::Click(click) = action else {
        panic!("expected click action");
    };
    assert!(matches!(
        click.interval.value(),
        Scriptable::Static { value: Some(value) } if *value == DurationValue::new(Duration::from_millis(250))
    ));
    assert!(matches!(
        click.duration.value(),
        Scriptable::Static { value: None }
    ));
}
