//! Selection: a list view in a dialog window of our own, with check boxes to select several
//! items.

use std::ptr;

use windows::{
    Win32::{
        Foundation::{HWND, RECT, SIZE},
        UI::{
            Controls::{
                LIST_VIEW_ITEM_STATE_FLAGS, LVCF_WIDTH, LVCOLUMNW, LVIF_TEXT, LVIR_BOUNDS,
                LVIS_FOCUSED, LVIS_SELECTED, LVIS_STATEIMAGEMASK, LVITEMW, LVM_ENSUREVISIBLE,
                LVM_GETCOLUMNWIDTH, LVM_GETITEMCOUNT, LVM_GETITEMRECT, LVM_GETITEMSTATE,
                LVM_GETNEXTITEM, LVM_INSERTCOLUMNW, LVM_INSERTITEMW, LVM_SETCOLUMNWIDTH,
                LVM_SETEXTENDEDLISTVIEWSTYLE, LVM_SETITEMCOUNT, LVM_SETITEMSTATE, LVN_ITEMACTIVATE,
                LVNI_FOCUSED, LVNI_SELECTED, LVS_EX_CHECKBOXES, LVS_EX_DOUBLEBUFFER,
                LVS_EX_FULLROWSELECT, LVS_NOCOLUMNHEADER, LVS_REPORT, LVS_SHOWSELALWAYS,
                LVS_SINGLESEL, LVSCW_AUTOSIZE, NMHDR, SetWindowTheme, WC_LISTVIEW,
            },
            HiDpi::GetSystemMetricsForDpi,
            WindowsAndMessaging::{GetClientRect, SM_CYEDGE, WS_EX_CLIENTEDGE},
        },
    },
    core::{HSTRING, PCWSTR, PWSTR, w},
};

use super::window::{self, CONTENT_WIDTH, Control, create_control, scale, send};
use crate::{Error, Result, SelectOptions};

/// Items shown at once, before the list is resized.
const VISIBLE_ITEMS: i32 = 10;

/// The state image of a checked item, `INDEXTOSTATEIMAGEMASK(2)`.
const CHECKED: LIST_VIEW_ITEM_STATE_FLAGS = LIST_VIEW_ITEM_STATE_FLAGS(2 << 12);

/// `wparam` for `LVM_GETNEXTITEM` to search from the start: -1.
const FROM_START: usize = usize::MAX;

fn next_item(control: HWND, flags: u32) -> Option<usize> {
    usize::try_from(send(
        control,
        LVM_GETNEXTITEM,
        FROM_START,
        flags.cast_signed() as isize,
    ))
    .ok()
}

fn set_state(
    control: HWND,
    index: usize,
    mask: LIST_VIEW_ITEM_STATE_FLAGS,
    state: LIST_VIEW_ITEM_STATE_FLAGS,
) {
    let item = LVITEMW {
        stateMask: mask,
        state,
        ..LVITEMW::default()
    };
    send(
        control,
        LVM_SETITEMSTATE,
        index,
        ptr::from_ref(&item) as isize,
    );
}

/// The indices of the checked items, or of the selected one.
fn selection(control: HWND, multiple: bool) -> Vec<usize> {
    if !multiple {
        return next_item(control, LVNI_SELECTED).into_iter().collect();
    }

    let count = usize::try_from(send(control, LVM_GETITEMCOUNT, 0, 0)).unwrap_or_default();
    (0..count)
        .filter(|&index| {
            let state = send(
                control,
                LVM_GETITEMSTATE,
                index,
                LVIS_STATEIMAGEMASK.0.cast_signed() as isize,
            );
            u32::try_from(state).unwrap_or_default() & LVIS_STATEIMAGEMASK.0 == CHECKED.0
        })
        .collect()
}

struct List<'a> {
    options: &'a SelectOptions,
    multiple: bool,
}

impl List<'_> {
    fn insert_items(&self, control: HWND) -> Result<()> {
        let column = LVCOLUMNW {
            mask: LVCF_WIDTH,
            ..LVCOLUMNW::default()
        };
        if send(
            control,
            LVM_INSERTCOLUMNW,
            0,
            ptr::from_ref(&column) as isize,
        ) < 0
        {
            return Err(Error::Backend(
                "the list's column could not be added".to_owned(),
            ));
        }

        send(control, LVM_SETITEMCOUNT, self.options.items.len(), 0);
        for (index, text) in self.options.items.iter().enumerate() {
            let text = HSTRING::from(text);
            let item = LVITEMW {
                mask: LVIF_TEXT,
                iItem: i32::try_from(index)
                    .map_err(|_| Error::InvalidOptions("there are too many items"))?,
                // The list view copies the text.
                pszText: PWSTR(text.as_ptr().cast_mut()),
                ..LVITEMW::default()
            };
            if send(control, LVM_INSERTITEMW, 0, ptr::from_ref(&item) as isize) < 0 {
                return Err(Error::Backend(
                    "an item could not be added to the list".to_owned(),
                ));
            }
        }
        Ok(())
    }

    /// Checks or selects the initial items, and focuses the first of them, which is where the
    /// keyboard starts.
    fn select_initial_items(&self, control: HWND) {
        let count = self.options.items.len();
        let mut initial = self
            .options
            .selected
            .iter()
            .copied()
            .filter(|&index| index < count);

        if self.multiple {
            for index in initial.clone() {
                set_state(control, index, LVIS_STATEIMAGEMASK, CHECKED);
            }
        }
        let focused = initial.next().unwrap_or(0);
        let state = LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0);
        set_state(control, focused, state, state);
    }
}

impl Control for List<'_> {
    fn create(&self, parent: HWND) -> Result<HWND> {
        let mut style = LVS_REPORT | LVS_NOCOLUMNHEADER | LVS_SHOWSELALWAYS;
        if !self.multiple {
            style |= LVS_SINGLESEL;
        }
        let control = create_control(parent, WC_LISTVIEW, style, WS_EX_CLIENTEDGE)?;
        // SAFETY: `control` was just created. Without the theme, the list looks older.
        _ = unsafe { SetWindowTheme(control, w!("Explorer"), PCWSTR::null()) };

        let mut extended = LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER;
        if self.multiple {
            extended |= LVS_EX_CHECKBOXES;
        }
        send(
            control,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            extended as usize,
            extended.cast_signed() as isize,
        );

        self.insert_items(control)?;
        self.select_initial_items(control);
        Ok(control)
    }

    fn size(&self, control: HWND, dpi: u32) -> SIZE {
        let mut item = RECT {
            left: LVIR_BOUNDS.cast_signed(),
            ..RECT::default()
        };
        send(
            control,
            LVM_GETITEMRECT,
            0,
            ptr::from_mut(&mut item) as isize,
        );
        let rows = i32::try_from(self.options.items.len())
            .unwrap_or(VISIBLE_ITEMS)
            .min(VISIBLE_ITEMS);
        // SAFETY: no preconditions.
        let border = unsafe { GetSystemMetricsForDpi(SM_CYEDGE, dpi) };
        SIZE {
            cx: scale(CONTENT_WIDTH, dpi),
            cy: (item.bottom - item.top) * rows + 2 * border,
        }
    }

    fn stretches(&self) -> bool {
        true
    }

    /// Makes the column fill the list, unless the items need more room, and keeps the focused item
    /// in view.
    fn resized(&self, control: HWND) {
        send(control, LVM_SETCOLUMNWIDTH, 0, LVSCW_AUTOSIZE as isize);
        let needed = send(control, LVM_GETCOLUMNWIDTH, 0, 0);
        let mut client = RECT::default();
        // SAFETY: `client` is writable.
        _ = unsafe { GetClientRect(control, &raw mut client) };
        if needed < client.right as isize {
            send(control, LVM_SETCOLUMNWIDTH, 0, client.right as isize);
        }

        if let Some(focused) = next_item(control, LVNI_FOCUSED) {
            send(control, LVM_ENSUREVISIBLE, focused, 0);
        }
    }

    fn can_accept(&self, control: HWND) -> bool {
        self.multiple || next_item(control, LVNI_SELECTED).is_some()
    }

    /// Double-clicking an item picks it, when picking one. When picking several, it checks it.
    fn accepts(&self, notification: &NMHDR) -> bool {
        !self.multiple && notification.code == LVN_ITEMACTIVATE
    }
}

/// Shows a selection list on the current thread. It is closed by the thread's `WM_QUIT`.
pub fn select(options: &SelectOptions, multiple: bool) -> Result<Option<Vec<usize>>> {
    window::show(
        &options.title,
        &options.text,
        &List { options, multiple },
        |control| selection(control, multiple),
    )
}
