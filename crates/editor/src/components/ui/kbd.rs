#![allow(clippy::absolute_paths)]

use leptos::prelude::*;
use leptos_ui::clx;

mod components {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    clx! {Kbd, kbd, "ui-kbd"}
    clx! {KbdGroup, kbd, "ui-kbd-group"}
}

pub use components::*;
