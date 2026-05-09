mod common;

#[test]
#[ignore = "interactive overlay selection"]
fn ask_rect_overlay() {
    e2e::selection_extension_bin();
    common::run("manual/ask_rect.ts").success();
}

#[test]
#[ignore = "interactive overlay selection"]
fn ask_position_overlay() {
    e2e::selection_extension_bin();
    common::run("manual/ask_position.ts").success();
}

#[test]
#[ignore = "drives the real mouse"]
fn mouse_action_overloads() {
    common::run("manual/mouse_actions.ts").success();
}
