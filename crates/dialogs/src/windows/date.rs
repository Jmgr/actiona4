//! Date picker: a month calendar in a dialog window of our own.

use std::ptr;

use jiff::civil::Date;
use windows::Win32::{
    Foundation::{HWND, RECT, SIZE, SYSTEMTIME},
    UI::{
        Controls::{
            MCM_GETCURSEL, MCM_GETMAXTODAYWIDTH, MCM_GETMINREQRECT, MCM_SETCURSEL, MONTHCAL_CLASS,
        },
        WindowsAndMessaging::WINDOW_EX_STYLE,
    },
};

use super::window::{self, Control, create_control, send};
use crate::{DateOptions, Error, Result};

/// `None` for years `SYSTEMTIME` cannot hold. The calendar also ignores dates before 1601, and
/// starts on today instead.
fn to_system_time(date: Date) -> Option<SYSTEMTIME> {
    Some(SYSTEMTIME {
        wYear: u16::try_from(date.year()).ok()?,
        wMonth: u16::try_from(date.month()).ok()?,
        wDayOfWeek: u16::try_from(date.weekday().to_sunday_zero_offset()).ok()?,
        wDay: u16::try_from(date.day()).ok()?,
        ..SYSTEMTIME::default()
    })
}

fn from_system_time(time: &SYSTEMTIME) -> Result<Date> {
    let invalid = || Error::Backend(format!("the calendar returned an invalid date: {time:?}"));
    Date::new(
        i16::try_from(time.wYear).map_err(|_| invalid())?,
        i8::try_from(time.wMonth).map_err(|_| invalid())?,
        i8::try_from(time.wDay).map_err(|_| invalid())?,
    )
    .map_err(|_| invalid())
}

struct Calendar(Option<Date>);

impl Control for Calendar {
    fn create(&self, parent: HWND) -> Result<HWND> {
        let control = create_control(parent, MONTHCAL_CLASS, 0, WINDOW_EX_STYLE(0))?;
        // The calendar starts on today otherwise.
        if let Some(time) = self.0.and_then(to_system_time) {
            send(control, MCM_SETCURSEL, 0, ptr::from_ref(&time) as isize);
        }
        Ok(control)
    }

    /// The size of a single month, which is wider if the text to go to today needs more room.
    fn size(&self, control: HWND, _dpi: u32) -> SIZE {
        let mut month = RECT::default();
        send(
            control,
            MCM_GETMINREQRECT,
            0,
            ptr::from_mut(&mut month) as isize,
        );
        let today = i32::try_from(send(control, MCM_GETMAXTODAYWIDTH, 0, 0)).unwrap_or_default();
        SIZE {
            cx: month.right.max(today),
            cy: month.bottom,
        }
    }
}

/// Shows a date picker on the current thread. It is closed by the thread's `WM_QUIT`.
pub fn date(options: &DateOptions) -> Result<Option<Date>> {
    window::show(
        &options.title,
        &options.text,
        &Calendar(options.value),
        |control| {
            let mut time = SYSTEMTIME::default();
            send(control, MCM_GETCURSEL, 0, ptr::from_mut(&mut time) as isize);
            from_system_time(&time)
        },
    )?
    .transpose()
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::{from_system_time, to_system_time};

    #[test]
    fn converts_dates() {
        let time = to_system_time(date(2026, 2, 3)).unwrap();
        assert_eq!((time.wYear, time.wMonth, time.wDay), (2026, 2, 3));
        // A Tuesday.
        assert_eq!(time.wDayOfWeek, 2);
        assert_eq!(from_system_time(&time).unwrap(), date(2026, 2, 3));

        assert!(to_system_time(date(-5, 1, 1)).is_none());
    }
}
