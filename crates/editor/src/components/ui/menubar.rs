//! Desktop-style menubar (File, Edit, ...).
//!
//! Behavior is driven entirely through DOM data attributes and a single set of
//! document-level listeners (see [`init_menubar_handlers`]), rather than per-item
//! event handlers. The components below just render the right markup; the
//! handlers read and mutate these attributes:
//!
//! - `[data-name="Menubar"]` — a menubar root; gains `data-active` while any of
//!   its menus is open.
//! - `[data-menubar-trigger=<id>]` — a top-level trigger button; its value is the
//!   `id` of the menu content it controls.
//! - `[data-menubar-content]` — a menu's content list; `data-state` is `open` or
//!   `closed`.
//! - `[data-menubar-close]` — an item that closes its menu when selected.
//! - `[data-menubar-mnemonic=<letter>]` — the `Alt`+letter / in-menu accelerator.
//! - `[data-highlighted="true"]` — the keyboard-navigation cursor within a menu.
//! - `[data-disabled]` — non-interactive item, skipped by clicks and navigation.
#![allow(clippy::absolute_paths)]

use std::cell::Cell;

use icons::{Check, ChevronRight};
use js_sys::{Function, Reflect};
use leptos::{context::Provider, prelude::*};
use leptos_ui::clx;
use tw_merge::tw_merge;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Document, Element, Event, HtmlElement, KeyboardEvent, Node};

use crate::components::hooks::use_random::use_random_id_for;

// Menubar behavior is delegated from the document rather than attaching a
// handler to every generated item. This flag keeps those document listeners
// registered once per browser thread.
thread_local! {
    static MENUBAR_INITIALIZED: Cell<bool> = const { Cell::new(false) };
}

// Purely presentational wrappers with no behavior; `clx!` generates a component
// that renders the given element with the given class.
mod components {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    clx! {MenubarGroup, ul, "ui-menubar-group"}
    clx! {MenubarLabel, div, "ui-menubar-label"}
    clx! {MenubarSubContent, ul, "ui-menubar-sub-content"}
}

pub use components::*;

#[component]
pub fn MenubarShortcut(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("ui-menubar-shortcut", class);
    view! {
        <span data-slot="menubar-shortcut" class=class>
            {children()}
        </span>
    }
}

#[component]
pub fn MenubarSeparator(#[prop(optional, into)] class: String) -> impl IntoView {
    let class = tw_merge!("ui-menubar-separator", class);

    view! { <li role="separator" data-name="MenubarSeparator" class=class /> }
}

#[component]
pub fn MenubarItem(
    /// Label text; an `&` before a character marks it as the mnemonic that
    /// triggers this item while the menu is open, and underlines it (see
    /// [`parse_mnemonic_label`]).
    #[prop(into)]
    label: Signal<String>,
    /// Trailing content rendered after the label, e.g. a [`MenubarShortcut`].
    #[prop(optional)]
    children: Option<Children>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_select: Option<Callback<()>>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let class = tw_merge!("ui-menubar-item", class);
    let mnemonic = move || parse_mnemonic_label(&label.get()).0.map(|c| c.to_string());
    view! {
        <li
            data-name="MenubarItem"
            class=class
            role="menuitem"
            tabindex="-1"
            data-menubar-close="true"
            data-menubar-mnemonic=mnemonic
            // Keep both a data attribute for styling/event guards and ARIA for
            // assistive technology.
            data-disabled=move || disabled.then_some("true")
            aria-disabled=move || disabled.to_string()
            on:click=move |_| {
                if disabled {
                    return;
                }

                if let Some(on_select) = on_select {
                    on_select.run(());
                }
            }
        >
            <span>{move || parse_mnemonic_label(&label.get()).1}</span>
            {children.map(|children| children())}
        </li>
    }
}

#[component]
pub fn MenubarCheckboxItem(
    children: Children,
    checked: RwSignal<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let class = tw_merge!("ui-menubar-choice-item", class);

    view! {
        <li
            data-name="MenubarCheckboxItem"
            class=class
            role="menuitemcheckbox"
            aria-checked=move || checked.get().to_string()
            // Disabled choice items stay visible but do not mutate their signal.
            data-disabled=move || disabled.then_some("true")
            aria-disabled=move || disabled.to_string()
            on:click=move |_| {
                if !disabled {
                    checked.update(|v| *v = !*v);
                }
            }
        >
            <span class="ui-menubar-check-slot">
                <Check class="ui-menubar-check-icon" />
            </span>
            {children()}
        </li>
    }
}

#[derive(Clone)]
struct MenubarRadioContext<T: Clone + PartialEq + Send + Sync + 'static> {
    value_signal: RwSignal<T>,
}

#[component]
pub fn MenubarRadioGroup<T>(children: Children, value: RwSignal<T>) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let ctx = MenubarRadioContext {
        value_signal: value,
    };

    view! {
        <Provider value=ctx>
            <ul data-name="MenubarRadioGroup" role="group" class="ui-menubar-radio-group">
                {children()}
            </ul>
        </Provider>
    }
}

#[component]
pub fn MenubarRadioItem<T>(
    children: Children,
    value: T,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let ctx = expect_context::<MenubarRadioContext<T>>();

    let value_for_check = value.clone();
    let value_for_click = value;
    let is_selected = move || ctx.value_signal.get() == value_for_check;

    let class = tw_merge!("ui-menubar-choice-item", class);

    view! {
        <li
            data-name="MenubarRadioItem"
            class=class
            role="menuitemradio"
            aria-checked=move || is_selected().to_string()
            // Disabled choice items stay visible but do not mutate their signal.
            data-disabled=move || disabled.then_some("true")
            aria-disabled=move || disabled.to_string()
            on:click=move |_| {
                if !disabled {
                    ctx.value_signal.set(value_for_click.clone());
                }
            }
        >
            <span class="ui-menubar-check-slot">
                <Check class="ui-menubar-check-icon" />
            </span>
            {children()}
        </li>
    }
}

#[derive(Clone)]
struct MenubarContext {
    menubar_id: String,
}

#[component]
pub fn Menubar(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    // The handlers are global and idempotent, so every Menubar instance can call
    // this safely.
    init_menubar_handlers();

    let menubar_id = use_random_id_for("menubar");
    let ctx = MenubarContext {
        menubar_id: menubar_id.clone(),
    };

    let class = tw_merge!("ui-menubar", class);

    // Once mounted, fail fast if two triggers in this menubar share a mnemonic.
    let root_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        let Some(root) = root_ref.get() else {
            return;
        };
        let root: Element = root.unchecked_into();
        assert_unique_mnemonics(
            query_all(&root, "[data-menubar-trigger][data-menubar-mnemonic]").into_iter(),
            "the menubar",
        );
    });

    view! {
        <Provider value=ctx>
            <div node_ref=root_ref data-name="Menubar" data-menubar-id=menubar_id class=class>
                {children()}
            </div>
        </Provider>
    }
}

#[derive(Clone)]
struct MenubarMenuContext {
    menu_id: String,
    menubar_id: String,
}

#[component]
pub fn MenubarMenu(children: Children) -> impl IntoView {
    let menubar_ctx = expect_context::<MenubarContext>();
    let menu_id = use_random_id_for("menubarmenu");

    let ctx = MenubarMenuContext {
        menu_id,
        menubar_id: menubar_ctx.menubar_id,
    };

    view! {
        <Provider value=ctx>
            <div data-name="MenubarMenu" class="ui-menubar-menu">
                {children()}
            </div>
        </Provider>
    }
}

/// Splits a trigger label into its rendered nodes and its mnemonic character.
///
/// An `&` marks the following character as the mnemonic: it is underlined and
/// becomes the `Alt`+letter accelerator. Use `&&` for a literal ampersand. This
/// keeps the label text and its accelerator a single source of truth, e.g.
/// `"&File"` underlines `F` and binds `Alt`+`F`.
fn parse_mnemonic_label(label: &str) -> (Option<char>, Vec<AnyView>) {
    let mut mnemonic = None;
    let mut nodes = Vec::new();
    let mut plain = String::new();
    let mut chars = label.chars();

    while let Some(c) = chars.next() {
        if c != '&' {
            plain.push(c);
            continue;
        }

        match chars.next() {
            Some('&') | None => plain.push('&'),
            Some(letter) => {
                if !plain.is_empty() {
                    nodes.push(view! { {std::mem::take(&mut plain)} }.into_any());
                }
                // The first marker wins if a label somehow has several.
                mnemonic.get_or_insert(letter.to_ascii_lowercase());
                nodes.push(
                    view! { <u class="ui-menubar-mnemonic">{letter.to_string()}</u> }.into_any(),
                );
            }
        }
    }

    if !plain.is_empty() {
        nodes.push(view! { {plain} }.into_any());
    }

    (mnemonic, nodes)
}

/// Panics if any two of `elements` declare the same mnemonic. Mnemonics within
/// one scope (a single menu, or the menubar's triggers) must be unique so that a
/// keystroke is never ambiguous; this surfaces the mistake at mount time.
fn assert_unique_mnemonics(elements: impl Iterator<Item = Element>, scope: &str) {
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for element in elements {
        let Some(key) = element.get_attribute("data-menubar-mnemonic") else {
            continue;
        };
        let label = element.text_content().unwrap_or_default();
        let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
        if let Some(previous) = seen.insert(key.clone(), label.clone()) {
            panic!(
                "Menubar: mnemonic '{key}' is used by both \"{previous}\" and \"{label}\" in {scope}; mnemonics must be unique within {scope}"
            );
        }
    }
}

/// The lowercased mnemonic character for a keyboard event, if its key is a
/// single alphanumeric character. Returns `None` for keys like `Enter` or `F1`.
fn mnemonic_key(event: &KeyboardEvent) -> Option<String> {
    let key = event.key();
    (key.chars().count() == 1 && key.chars().all(|c| c.is_ascii_alphanumeric()))
        .then(|| key.to_ascii_lowercase())
}

#[component]
pub fn MenubarTrigger(
    /// Label text; an `&` before a character marks it as the `Alt`+letter
    /// mnemonic and underlines it (see [`parse_mnemonic_label`]).
    #[prop(into)]
    label: Signal<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let ctx = expect_context::<MenubarMenuContext>();
    let class = tw_merge!("ui-menubar-trigger", class);

    let mnemonic = move || parse_mnemonic_label(&label.get()).0.map(|c| c.to_string());
    // Descriptive only; the binding itself is handled by the document listener.
    let keyshortcuts = move || mnemonic().map(|m| format!("Alt+{}", m.to_uppercase()));

    view! {
        <button
            type="button"
            data-name="MenubarTrigger"
            data-menubar-trigger=ctx.menu_id
            data-menubar-id=ctx.menubar_id
            data-menubar-mnemonic=mnemonic
            aria-keyshortcuts=keyshortcuts
            class=class
            aria-expanded="false"
        >
            {move || parse_mnemonic_label(&label.get()).1}
        </button>
    }
}

#[component]
pub fn MenubarContent(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let ctx = expect_context::<MenubarMenuContext>();

    let class = tw_merge!("ui-menubar-content", class);

    let menu_id = ctx.menu_id;

    // Once mounted, fail fast if two items in this menu share a mnemonic.
    let content_ref = NodeRef::<leptos::html::Ul>::new();
    Effect::new(move |_| {
        let Some(content) = content_ref.get() else {
            return;
        };
        let content: Element = content.unchecked_into();
        // Only this menu's own items, not those of any nested submenu.
        let items = query_all(&content, "[data-menubar-mnemonic]")
            .into_iter()
            .filter(|item| {
                closest(item, "[data-menubar-content]")
                    .is_some_and(|owner| owner.is_same_node(Some(&content)))
            });
        assert_unique_mnemonics(items, "the same menu");
    });

    view! {
        <ul
            node_ref=content_ref
            data-name="MenubarContent"
            data-menubar-content=""
            class=class
            id=menu_id
            data-state="closed"
        >
            {children()}
        </ul>
    }
}

#[component]
pub fn MenubarSub(children: Children) -> impl IntoView {
    clx! {MenubarSubRoot, li, "ui-menubar-sub-trigger"}

    view! { <MenubarSubRoot>{children()}</MenubarSubRoot> }
}

#[component]
pub fn MenubarSubTrigger(
    children: Children,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let class = tw_merge!("ui-menubar-sub-trigger-content", class);

    view! {
        <span data-name="MenubarSubTrigger" class=class>
            <span class="ui-menubar-sub-trigger-label">{children()}</span>
            <ChevronRight class="ui-menubar-sub-trigger-icon" />
        </span>
    }
}

#[component]
pub fn MenubarSubItem(
    children: Children,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let class = tw_merge!("ui-menubar-sub-item", class);

    view! {
        <li
            data-name="MenubarSubItem"
            class=class
            data-menubar-close="true"
            data-disabled=move || disabled.then_some("true")
            aria-disabled=move || disabled.to_string()
        >
            {children()}
        </li>
    }
}

fn init_menubar_handlers() {
    if MENUBAR_INITIALIZED.with(|initialized| initialized.replace(true)) {
        return;
    }

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };

    // Click handling covers three cases: toggling a trigger, selecting an item
    // that should close its menu, and closing active menus from outside clicks.
    let click_document = document.clone();
    let click = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event| {
        if let Some(trigger) = closest_event_target(&event, "[data-menubar-trigger]") {
            event.stop_propagation();

            if let Some(menu) = menu_for_trigger(&click_document, &trigger) {
                if is_open(&menu) {
                    close_menu(&menu);
                } else {
                    open_menu(&menu, &trigger);
                }
            }

            return;
        }

        if let Some(close_item) = closest_event_target(&event, "[data-menubar-close]") {
            // Disabled items should not fire selection side effects and should
            // also leave the menu open.
            if close_item.has_attribute("data-disabled") {
                return;
            }

            if let Some(menu) = closest(&close_item, "[data-menubar-content]") {
                close_menu(&menu);
            }

            return;
        }

        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Node>().ok())
        else {
            return;
        };

        for root in document_query_all(&click_document, r#"[data-name="Menubar"][data-active]"#) {
            if !root
                .dyn_ref::<Node>()
                .is_some_and(|root| root.contains(Some(&target)))
            {
                close_all(&root);
            }
        }
    }));
    _ = document.add_event_listener_with_callback("click", click.as_ref().unchecked_ref());
    click.forget();

    // Once a menubar is active, hovering another trigger switches to that menu,
    // matching desktop application menubar behavior.
    let mouseover_document = document.clone();
    let mouseover = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event| {
        let Some(trigger) = closest_event_target(&event, "[data-menubar-trigger]") else {
            return;
        };
        let Some(menu) = menu_for_trigger(&mouseover_document, &trigger) else {
            return;
        };
        let Some(root) = menu_root(&trigger) else {
            return;
        };

        if root.has_attribute("data-active") && !is_open(&menu) {
            open_menu(&menu, &trigger);
        }
    }));
    _ = document.add_event_listener_with_callback("mouseover", mouseover.as_ref().unchecked_ref());
    mouseover.forget();

    // Keyboard access: Alt + a menu's mnemonic toggles that menu, and Escape
    // closes every active menubar even if focus is inside a menu item.
    let keydown_document = document.clone();
    let keydown = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(
        move |event: KeyboardEvent| {
            // Branches are ordered most-specific first and each returns once it
            // handles the event: in-menu navigation, then Alt+mnemonic to open a
            // menu, then a bare mnemonic to pick an item, then Escape to close.

            // Arrow keys and Enter navigate the currently open menu.
            if let Some(content) = open_menu_content(&keydown_document) {
                match event.key().as_str() {
                    "ArrowDown" => {
                        event.prevent_default();
                        move_highlight(&content, 1);
                        return;
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        move_highlight(&content, -1);
                        return;
                    }
                    "ArrowRight" => {
                        event.prevent_default();
                        open_sibling_menu(&keydown_document, &content, 1);
                        return;
                    }
                    "ArrowLeft" => {
                        event.prevent_default();
                        open_sibling_menu(&keydown_document, &content, -1);
                        return;
                    }
                    "Enter" => {
                        if let Some(item) = highlighted_item(&content)
                            .and_then(|i| i.dyn_into::<HtmlElement>().ok())
                        {
                            event.prevent_default();
                            item.click();
                            return;
                        }
                    }
                    _ => {}
                }
            }

            let mnemonic = mnemonic_key(&event);

            // Alt + a top-level menu's mnemonic toggles that menu.
            if event.alt_key()
                && !event.ctrl_key()
                && !event.meta_key()
                && let Some(mnemonic) = &mnemonic
            {
                let selector =
                    format!(r#"[data-menubar-trigger][data-menubar-mnemonic="{mnemonic}"]"#);
                if let Some(trigger) = document_query_all(&keydown_document, &selector)
                    .into_iter()
                    .next()
                    && let Some(menu) = menu_for_trigger(&keydown_document, &trigger)
                {
                    event.prevent_default();
                    if is_open(&menu) {
                        close_menu(&menu);
                    } else {
                        open_menu(&menu, &trigger);
                        // Opening via the keyboard pre-selects the first
                        // non-disabled item, like arrowing between menus.
                        move_highlight(&menu, 1);
                    }
                    return;
                }
            }

            // While a menu is open, the bare mnemonic key activates its item.
            if !event.alt_key()
                && !event.ctrl_key()
                && !event.meta_key()
                && let Some(mnemonic) = &mnemonic
            {
                let selector = format!(
                    r#"[data-menubar-content][data-state="open"] [data-menubar-mnemonic="{mnemonic}"]:not([data-disabled])"#
                );
                if let Some(item) = document_query_all(&keydown_document, &selector)
                    .into_iter()
                    .find_map(|item| item.dyn_into::<HtmlElement>().ok())
                {
                    event.prevent_default();
                    // Reuse the click path so selection and close behave
                    // exactly as a pointer activation would.
                    item.click();
                    return;
                }
            }

            if event.key() == "Escape" {
                let roots =
                    document_query_all(&keydown_document, r#"[data-name="Menubar"][data-active]"#);
                if !roots.is_empty() {
                    event.prevent_default();
                    for root in roots {
                        close_all(&root);
                    }
                }
            }
        },
    ));
    _ = document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref());
    keydown.forget();
}

fn closest_event_target(event: &Event, selector: &str) -> Option<Element> {
    let target = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())?;

    closest(&target, selector)
}

fn closest(element: &Element, selector: &str) -> Option<Element> {
    element.closest(selector).ok().flatten()
}

fn is_open(menu: &Element) -> bool {
    menu.get_attribute("data-state").as_deref() == Some("open")
}

fn menu_root(element: &Element) -> Option<Element> {
    closest(element, r#"[data-name="Menubar"]"#)
}

// A trigger and its menu are linked by id: the trigger's `data-menubar-trigger`
// attribute holds the `id` of its menu content element. These two helpers walk
// that link in each direction.
fn menu_for_trigger(document: &Document, trigger: &Element) -> Option<Element> {
    let menu_id = trigger.get_attribute("data-menubar-trigger")?;

    document.get_element_by_id(&menu_id)
}

fn trigger_for_menu(menu: &Element) -> Option<Element> {
    let menu_id = menu.get_attribute("id")?;
    let root = menu_root(menu)?;

    query_all(&root, "[data-menubar-trigger]")
        .into_iter()
        .find(|trigger| trigger.get_attribute("data-menubar-trigger").as_deref() == Some(&menu_id))
}

fn update_position(menu: &Element, trigger: &Element) {
    // Menus are fixed-position overlays. Prefer opening below the trigger, but
    // flip above when there is not enough space below and there is room above.
    let trigger_rect = trigger.get_bounding_client_rect();
    let menu_rect = menu.get_bounding_client_rect();
    let inner_height = web_sys::window()
        .and_then(|window| window.inner_height().ok())
        .and_then(|height| height.as_f64())
        .unwrap_or_default();
    let space_below = inner_height - trigger_rect.bottom();
    let space_above = trigger_rect.top();

    let Some(menu) = menu.dyn_ref::<HtmlElement>() else {
        return;
    };
    let style = menu.style();

    if space_above >= menu_rect.height() && space_below < menu_rect.height() {
        _ = style.set_property(
            "top",
            &format!("{}px", trigger_rect.top() - menu_rect.height() - 4.0),
        );
        _ = style.set_property("transform-origin", "left bottom");
    } else {
        _ = style.set_property("top", &format!("{}px", trigger_rect.bottom() + 4.0));
        _ = style.set_property("transform-origin", "left top");
    }

    _ = style.set_property("left", &format!("{}px", trigger_rect.left()));
}

fn close_menu(menu: &Element) {
    let root = menu_root(menu);
    let trigger = trigger_for_menu(menu);

    _ = menu.set_attribute("data-state", "closed");
    // A reopened menu should start with nothing highlighted.
    clear_highlight(menu);
    if let Some(menu) = menu.dyn_ref::<HtmlElement>() {
        _ = menu.style().set_property("visibility", "");
    }
    if let Some(trigger) = trigger {
        _ = trigger.set_attribute("aria-expanded", "false");
    }

    let any_open = root.as_ref().is_some_and(|root| {
        query_all(root, "[data-menubar-content]")
            .iter()
            .any(is_open)
    });

    // Scroll is locked while any menu in this menubar is open. Only unlock after
    // the last one closes.
    if !any_open {
        if let Some(root) = root {
            _ = root.remove_attribute("data-active");
        }
        scroll_lock("unlock", Some(200.0));
    }
}

fn close_all(root: &Element) {
    for menu in query_all(root, "[data-menubar-content]") {
        close_menu(&menu);
    }

    for trigger in query_all(root, "[data-menubar-trigger]") {
        _ = trigger.set_attribute("aria-expanded", "false");
    }

    _ = root.remove_attribute("data-active");
    scroll_lock("unlock", Some(200.0));
}

fn open_menu(menu: &Element, trigger: &Element) {
    let Some(root) = menu_root(trigger) else {
        return;
    };

    // A menubar shows one top-level menu at a time.
    for other_menu in query_all(&root, "[data-menubar-content]") {
        if !other_menu.is_same_node(Some(menu)) && is_open(&other_menu) {
            close_menu(&other_menu);
        }
    }

    _ = root.set_attribute("data-active", "true");
    _ = trigger.set_attribute("aria-expanded", "true");
    _ = menu.set_attribute("data-state", "open");

    if let Some(menu) = menu.dyn_ref::<HtmlElement>() {
        _ = menu.style().set_property("visibility", "hidden");
        _ = menu.offset_height();
    }

    update_position(menu, trigger);

    if let Some(menu) = menu.dyn_ref::<HtmlElement>() {
        _ = menu.style().set_property("visibility", "visible");
    }

    // Move focus to the trigger so a freshly opened menu starts with nothing
    // highlighted, even if a previous menu left focus on one of its items.
    if let Some(trigger) = trigger.dyn_ref::<HtmlElement>() {
        _ = trigger.focus();
    }

    scroll_lock("lock", None);
}

/// The currently open top-level menu content, if any.
fn open_menu_content(document: &Document) -> Option<Element> {
    document_query_all(document, r#"[data-menubar-content][data-state="open"]"#)
        .into_iter()
        .next()
}

/// Selectable (non-disabled) items of a menu, in DOM order.
fn menu_items(content: &Element) -> Vec<Element> {
    query_all(content, r#"[role^="menuitem"]:not([data-disabled])"#)
}

fn clear_highlight(content: &Element) {
    for item in query_all(content, "[data-highlighted]") {
        _ = item.remove_attribute("data-highlighted");
    }
}

fn highlighted_item(content: &Element) -> Option<Element> {
    query_all(content, r#"[data-highlighted="true"]"#)
        .into_iter()
        .next()
}

/// Moves the highlight by `step` (e.g. `1` for down, `-1` for up), wrapping
/// around the ends and skipping disabled items. With nothing highlighted yet,
/// moving down lands on the first item and moving up on the last.
fn move_highlight(content: &Element, step: isize) {
    let items = menu_items(content);
    if items.is_empty() {
        return;
    }

    let next = match highlighted_item(content).and_then(|current| {
        items
            .iter()
            .position(|item| item.is_same_node(Some(&current)))
    }) {
        Some(index) => wrapped_index(index, items.len(), step),
        None if step >= 0 => 0,
        None => items.len() - 1,
    };

    clear_highlight(content);
    _ = items[next].set_attribute("data-highlighted", "true");
    // Focus keeps the item visible in scrollable menus and routes Enter to it.
    if let Some(item) = items[next].dyn_ref::<HtmlElement>() {
        _ = item.focus();
    }
}

/// Opens the menu `step` positions from the open one (e.g. `1` for the menu to
/// the right), wrapping around the menubar.
fn open_sibling_menu(document: &Document, content: &Element, step: isize) {
    let Some(root) = menu_root(content) else {
        return;
    };
    let triggers = query_all(&root, "[data-menubar-trigger]");
    let Some(current) = trigger_for_menu(content)
        .and_then(|trigger| triggers.iter().position(|t| t.is_same_node(Some(&trigger))))
    else {
        return;
    };

    let next = wrapped_index(current, triggers.len(), step);
    let trigger = &triggers[next];
    if let Some(menu) = menu_for_trigger(document, trigger) {
        open_menu(&menu, trigger);
        // Unlike opening via mouse or Alt+letter, arrowing between menus is a
        // keyboard gesture, so pre-select the first non-disabled item.
        move_highlight(&menu, 1);
    }
}

fn query_all(root: &Element, selector: &str) -> Vec<Element> {
    root.query_selector_all(selector)
        .ok()
        .as_ref()
        .map(elements_from_node_list)
        .unwrap_or_default()
}

fn document_query_all(document: &Document, selector: &str) -> Vec<Element> {
    document
        .query_selector_all(selector)
        .ok()
        .as_ref()
        .map(elements_from_node_list)
        .unwrap_or_default()
}

fn elements_from_node_list(node_list: &web_sys::NodeList) -> Vec<Element> {
    (0..node_list.length())
        .filter_map(|index| node_list.item(index))
        .filter_map(|node| node.dyn_into::<Element>().ok())
        .collect()
}

const fn wrapped_index(index: usize, len: usize, step: isize) -> usize {
    let distance = step.unsigned_abs() % len;

    if step.is_negative() {
        if distance > index {
            len - (distance - index)
        } else {
            index - distance
        }
    } else {
        let remaining = len - index;
        if distance >= remaining {
            distance - remaining
        } else {
            index + distance
        }
    }
}

fn scroll_lock(method: &str, arg: Option<f64>) {
    // ScrollLock is provided by the page shell. Treat it as optional so this UI
    // component can still render in tests or alternate hosts.
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(scroll_lock) = Reflect::get(&window, &JsValue::from_str("ScrollLock")) else {
        return;
    };
    let Ok(method) = Reflect::get(&scroll_lock, &JsValue::from_str(method))
        .and_then(|value| value.dyn_into::<Function>())
    else {
        return;
    };

    match arg {
        Some(arg) => {
            _ = method.call1(&scroll_lock, &JsValue::from_f64(arg));
        }
        None => {
            _ = method.call0(&scroll_lock);
        }
    }
}
