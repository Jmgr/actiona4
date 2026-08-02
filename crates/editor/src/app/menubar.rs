#![allow(clippy::absolute_paths)]

use leptos::{context::Provider, ev, prelude::*};
use leptos_fluent::move_tr;
use strum::{EnumIter, IntoEnumIterator};

use crate::components::ui::{
    kbd::{Kbd, KbdGroup},
    menubar::{
        Menubar, MenubarContent, MenubarGroup, MenubarItem, MenubarMenu, MenubarSeparator,
        MenubarShortcut, MenubarTrigger,
    },
};

#[derive(Clone, Copy, EnumIter)]
enum FileMenuAction {
    Open,
    Save,
    SaveAs,
    Exit,
}

impl FileMenuAction {
    fn label(self) -> Signal<String> {
        use FileMenuAction::*;
        match self {
            Open => move_tr!("menu-file-open"),
            Save => move_tr!("menu-file-save"),
            SaveAs => move_tr!("menu-file-save-as"),
            Exit => move_tr!("menu-file-exit"),
        }
    }

    const fn shortcut(self) -> &'static [&'static str] {
        use FileMenuAction::*;
        match self {
            Open => &["Ctrl", "O"],
            Save => &["Ctrl", "S"],
            SaveAs => &["Ctrl", "Alt", "S"],
            Exit => &["Ctrl", "W"],
        }
    }

    /// Whether `event` matches this action's shortcut, derived from
    /// [`shortcut`](Self::shortcut) so the keys are never duplicated.
    fn matches(self, event: &ev::KeyboardEvent) -> bool {
        let keys = self.shortcut();
        // Modifiers come first; the final entry is the actual key.
        let Some(key) = keys.last() else {
            return false;
        };
        event.ctrl_key() == keys.contains(&"Ctrl")
            && event.alt_key() == keys.contains(&"Alt")
            && event.shift_key() == keys.contains(&"Shift")
            && event.meta_key() == keys.contains(&"Meta")
            && event.key().eq_ignore_ascii_case(key)
    }
}

#[derive(Clone, Copy)]
struct FileMenuContext {
    on_select: Callback<FileMenuAction>,
}

#[component]
fn AppMenuItem(action: FileMenuAction, #[prop(optional)] disabled: bool) -> impl IntoView {
    let ctx = expect_context::<FileMenuContext>();
    let on_select = Callback::new(move |()| ctx.on_select.run(action));

    view! {
        <MenubarItem label=action.label() on_select disabled>
            <MenubarShortcut>
                <KbdGroup>
                    {action
                        .shortcut()
                        .iter()
                        .enumerate()
                        .map(|(index, key)| {
                            view! {
                                {if index > 0 { Some(view! { <span>"+"</span> }) } else { None }}
                                <Kbd>{*key}</Kbd>
                            }
                        })
                        .collect_view()}
                </KbdGroup>
            </MenubarShortcut>
        </MenubarItem>
    }
}

#[component]
pub fn AppMenubar(on_exit: fn()) -> impl IntoView {
    use FileMenuAction::*;
    let on_file_select = Callback::new(move |action: FileMenuAction| match action {
        Open => {
            leptos::logging::log!("File > Open");
        }
        Save => {
            leptos::logging::log!("File > Save");
        }
        SaveAs => {
            leptos::logging::log!("File > Save As...");
        }
        Exit => {
            on_exit();
        }
    });

    let handle = window_event_listener(ev::keydown, move |event| {
        if let Some(action) = FileMenuAction::iter().find(|action| action.matches(&event)) {
            event.prevent_default();
            on_file_select.run(action);
        }
    });
    on_cleanup(move || handle.remove());

    view! {
        <Menubar>
            <Provider value=FileMenuContext {
                on_select: on_file_select,
            }>
                <MenubarMenu>
                    <MenubarTrigger label=move_tr!("menu-file") />
                    <MenubarContent>
                        <MenubarGroup>
                            <AppMenuItem action=Open />
                            <AppMenuItem action=Save />
                            <AppMenuItem action=SaveAs />
                            <MenubarSeparator />
                            <AppMenuItem action=Exit />
                        </MenubarGroup>
                    </MenubarContent>
                </MenubarMenu>
            </Provider>
            // A second menu with fictional entries to exercise keyboard
            // navigation (arrows, wrap-around, and the disabled "Cut" is
            // skipped).
            <MenubarMenu>
                <MenubarTrigger label=move_tr!("menu-edit") />
                <MenubarContent>
                    <MenubarGroup>
                        <MenubarItem label=move_tr!("menu-edit-undo") on_select=log_select("Edit > Undo") />
                        <MenubarItem label=move_tr!("menu-edit-redo") on_select=log_select("Edit > Redo") />
                        <MenubarSeparator />
                        <MenubarItem label=move_tr!("menu-edit-cut") disabled=true />
                        <MenubarItem label=move_tr!("menu-edit-copy") on_select=log_select("Edit > Copy") />
                        <MenubarItem label=move_tr!("menu-edit-paste") on_select=log_select("Edit > Paste") />
                    </MenubarGroup>
                </MenubarContent>
            </MenubarMenu>
        </Menubar>
    }
}

fn log_select(message: &'static str) -> Callback<()> {
    Callback::new(move |()| leptos::logging::log!("{message}"))
}
