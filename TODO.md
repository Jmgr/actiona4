TODO:
- select a rectangle UI

Find Image:
 1) search one or search multiple
 2) search for multiple templates (in parallel), label them
 3) track an item (post 1.0)
 4) UI to test parameters and display results on screen (transparent target icon?)
 
 - window management: active-win-pos-rs
 
 - High DPI?

 Mouse:
- record
- drag and drop?

Update
- show notification if new version

Notification:
- add a "permanent" mode that sets .hint(Hint::Resident(true)) and .timeout(Timeout::Never)
- add a "close" function

- check we don't await without a cancellation token

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

You basically have two different worlds here:

* **Windows toast notifications** can be made *self-contained* (click → OS opens a URL / protocol) so it still works after `actiona-run` exits.
* **Linux/FreeDesktop notifications** actions are *callbacks to your app over D-Bus*, so if your process is gone there’s nothing left to receive the click.

Below are the options that actually behave well for a short-lived CLI.

---

## Windows: make the toast open a URL (works even after exit)

On Windows, the clean solution is: **use “protocol activation”** for the toast body and/or buttons.

* You can set `activationType="protocol"` and `arguments="https://…"` to open the default browser. ([Microsoft Learn][1])
* For classic Win32 “desktop apps”, Microsoft even notes that when you use the “stub CLSID” approach (to get Action Center persistence), **you can only use protocol activation**. ([Microsoft Learn][2])

So your “Download” button can simply be an `https://github.com/.../releases/...` link. No running process needed.

### “Ignore” on Windows

You have two workable patterns:

1. **Make “Ignore” also a protocol** but to a *custom scheme*, e.g. `actiona://ignore?version=1.2.3`.

   * Register that custom protocol in the registry so Windows launches your app (or a tiny helper) with the argument when clicked. ([Sipgate][3])
   * When launched, you just write the ignore marker/config and exit immediately.

2. Skip the button and instead do “Ignore this version” inside the app via a CLI flag (`actiona-run --ignore-update 1.2.3`) that users can run later. (Less UX, but simplest.)

Also note you can choose a toast “scenario” like `reminder` to keep it visible longer (but that can be annoying for a CLI tool, so I’d only do it if the user has opted in). ([Microsoft Learn][1])

---

## Linux: actions require your app to still be running

Under the Desktop Notifications spec, the server emits an “ActionInvoked” signal back to the client (your process) when the user clicks the notification or a button. If your app is already gone, there’s nothing to receive that signal. ([Galago Project][4])

Also, timeouts aren’t dependable: you can request “never expire” (`expire_timeout = 0`), but servers vary and may still behave differently. ([Freedesktop Specifications][5])

### The pragmatic Linux approach for a short-lived CLI

**Don’t rely on action buttons.** Instead:

1. **Put the download URL in the notification body** (as plain text).
   Many shells will auto-link it even if they don’t support explicit `<a href>` markup. For example, GNOME Shell doesn’t advertise `body-hyperlinks`, but it will still convert plain URLs into clickable links. ([GNOME Discourse][6])

2. Optionally, if the server reports it supports markup/hyperlinks (`body-markup` / `body-hyperlinks`), you *can* send `<a href="…">Download</a>`; but you **must** feature-detect because some servers will show raw tags. ([Freedesktop Specifications][7])

3. Always add a **terminal fallback**: print one concise line like
   `Update available: 1.2.3 → https://…`
   This avoids “notification vanished, user lost the link”.

### If you really want “Ignore” on Linux without staying alive

Your only robust way (similar to Windows) is to make the click **launch something** *independently of your running process*:

* Register a custom URL scheme handler via a `.desktop` entry (`MimeType=x-scheme-handler/actiona;`), so `actiona://ignore?version=…` launches `actiona-run --handle-url ...`.

But the catch is: **the notification daemon must render that as a clickable link**. With GNOME, you’ll likely need to include the literal `actiona://…` text and hope it’s linkified the same way as `https://…` (not guaranteed across DEs). The spec allows hyperlinks, but support is explicitly capability-based. ([Freedesktop Specifications][7])

So in practice I’d still recommend: **Linux notification = link to download page**, ignore handled inside your normal config/CLI commands.

---

## A design that works well cross-platform (recommended)

**Notification content:**

* Title: “Actiona update available (1.2.3)”
* Body: “Download: https://…  (or run: actiona-run --open-release)”
* Windows: add two buttons using protocol activation:

  * **Download** → `https://…`
  * **Ignore** → `actiona://ignore?version=1.2.3` (custom protocol)
* Linux: no buttons; just the URL in the body.

**CLI behavior:**

* Never block waiting for notification interaction.
* Store “latest seen update” and “ignored versions” locally; show the notification at most once per version (or with a cooldown).

This avoids the “clicked later but nothing happens” failure mode entirely, while still giving Windows users the nice buttons.

If you tell me what you use today for notifications on Linux (e.g. `notify_rust` + `zbus`), and on Windows (e.g. `winrt_toast_reborn`), I can map the above to the exact APIs/payload you need.

[1]: https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/toast-schema?utm_source=chatgpt.com "Toast content schema - Windows apps"
[2]: https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/toast-desktop-apps?utm_source=chatgpt.com "Activating toast notifications from desktop apps"
[3]: https://www.sipgate.de/blog/how-to-create-native-notifications-with-action-buttons-on-windows-for-your-electron-app?utm_source=chatgpt.com "Native Windows Notifications with Action Buttons for your ..."
[4]: https://galago-project.org/specs/notification/0.9/x408.html?utm_source=chatgpt.com "9. D-BUS Protocol - Galago"
[5]: https://specifications.freedesktop.org/notification/1.3/basic-design.html?utm_source=chatgpt.com "Basic Design | Desktop Notifications Specification"
[6]: https://discourse.gnome.org/t/hyperlink-not-working-with-notify-send/21259?utm_source=chatgpt.com "Hyperlink not working with `notify-send` - Desktop"
[7]: https://specifications.freedesktop.org/notification/latest-single?utm_source=chatgpt.com "Desktop Notifications Specification"
