use leptos::prelude::*;
use leptos_fluent::leptos_fluent;

#[component]
pub fn I18nProvider(children: Children) -> impl IntoView {
    leptos_fluent! {
        children: children(),
        locales: "./locales",
        default_language: "en-US",
        sync_html_tag_lang: true,
        sync_html_tag_dir: true,
        check_translations: true,
    }
}
