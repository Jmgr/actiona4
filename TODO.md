TODO:
- process creation
- select a rectangle UI

Notifications:
- replace hand rolled AUMID Windows code with crate winrt_toast_reborn
- notify_rust for X11
- see features/notifications.md

Find Image:
 1) search one or search multiple
 2) search for multiple templates (in parallel), label them
 3) track an item (post 1.0)
 4) UI to test parameters and display results on screen (transparent target icon?)

 Mouse:
- record
- drag and drop?

- allow sleep to accept a string: sleep("10s") -- maybe add a DurationLike?


// TODO: check all errors
// TODO: check all token cancellation return a Cancelled error
// TODO: check all unwraps
// TODO: display a tray icon, enabled by default when waitAtEnd is true
// TODO: enigo::set_dpi_awareness()
/*
Note that the top-left hand corner of the desktop is not necessarily the same as the screen.
If the user uses a desktop with multiple monitors, the top-left hand corner of the desktop is
the top-left hand corner of the main monitor on Windows and macOS or the top-left of the
leftmost monitor on X11.
*/
/*
use windows_sys::Win32::Globalization::CP_UTF8;
use windows_sys::Win32::System::Console::SetConsoleOutputCP;

unsafe {
    SetConsoleOutputCP(CP_UTF8);
}
*/
/*
You are running actiona4-run version 0.1.0, latest version is 1.0.1,
released 3d ago.
*/
// TODO: 3d ago? Oo
// TODO: Maybe remove Arc<Foo> and make Foo clonable directly
