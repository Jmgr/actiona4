use leptos::prelude::*;

use crate::app::menubar::AppMenubar;

mod menubar;

#[allow(clippy::absolute_paths)]
#[component]
pub fn App(on_exit: fn()) -> impl IntoView {
    view! {
        <div class="app-shell">
            <AppMenubar on_exit />
        </div>
    }
}
