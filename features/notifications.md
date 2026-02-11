**Planning final Windows/Linux feature comparison**

`winrt_toast_reborn` vs `notify-rust` (ignoring macOS), based on current docs:

| Feature | `winrt_toast_reborn` | `notify-rust` | Tag |
|---|---|---|---|
| OS scope | Windows only | Linux/XDG + Windows (limited API on Windows) | `Windows-only` / mixed |
| Basic notification send | Yes | Yes (`summary`, `body`, `show`) | `Windows + Linux` |
| Timeout/lifetime | Yes (`duration`, `expires_in`) | Yes (`timeout`) | `Windows + Linux` |
| Rich text layout | Yes (`text1/text2/text3`, `header`) | Limited (`summary/body/subtitle`) | Mostly `Windows-only` richness |
| Buttons/actions | Yes (`Action`, activation settings) | Linux/XDG only (`action`) | `Linux-only` in `notify-rust` |
| Text input + selection controls in toast | Yes (`Input`, `Selection`) | No equivalent on Windows/Linux API surface | `Windows-only` |
| Images as toast content elements | Yes (`Image`, placement/crop) | Linux/XDG-only via `images` feature | split (`Windows-only` vs `Linux-only`) |
| Audio control/sounds | Yes (`Audio`, `Sound`, looping/silent) | Via hints/sound wrappers on XDG path | mostly `Windows-only` explicit API |
| Scenario types (alarm/reminder/call/urgent) | Yes (`Scenario`) | No | `Windows-only` |
| Activation/dismiss/failure callbacks | Yes (`on_activated`, `on_dismissed`, `on_failed`) | Linux/XDG only (`wait_for_action`, `on_close`) | split |
| Update/close existing notifications | Yes (`remove`, `remove_group`, `remove_grouped_tag`, `clear`) | Linux/XDG only (`close`, `update`, `id`) | split |
| App registration / AUM ID handling | Yes (`register`, `unregister`, custom AUM ID) | No | `Windows-only` |
| Server capability queries | No | Linux/XDG only (`get_capabilities`, `get_server_information`) | `Linux-only` |

Cargo feature flags:

- `winrt_toast_reborn`: no Cargo feature flags.
- `notify-rust`: `default`, `z`, `async`, `serde`, `zbus`, `chrono`, `d`, `d_vendored`, `dbus`, `debug_namespace`, `env_logger`, `image`, `images`, `lazy_static`.
- Linux/XDG-focused flags in `notify-rust`: `images`/`image`/`lazy_static`, `d`/`d_vendored`/`dbus`/`zbus`.

Sources:
- https://docs.rs/winrt-toast-reborn/latest/winrt_toast_reborn/
- https://docs.rs/winrt-toast-reborn/latest/winrt_toast_reborn/struct.Toast.html
- https://docs.rs/winrt-toast-reborn/latest/winrt_toast_reborn/struct.ToastManager.html
- https://docs.rs/winrt-toast-reborn/latest/src/winrt_toast_reborn/toast.rs.html
- https://docs.rs/notify-rust/latest/src/notify_rust/lib.rs.html
- https://docs.rs/crate/notify-rust/latest/features
- https://docs.rs/crate/winrt-toast-reborn/0.3.8/features

**Summarizing AUMID use and multi-instance strategy**

Using `ToastManager::POWERSHELL_AUM_ID` is fine for quick tests, but has real tradeoffs in production.

- Notifications are attributed to **PowerShell**, not Actiona (name/icon/settings bucket/history).
- If users disable or tune PowerShell notifications, your notifications are affected too.
- `clear/remove` operations under that AUMID can interfere with PowerShell’s own notifications.
- The crate itself labels it **experimental**.

Yes, Actiona should have its own AUMID.

- Pick one stable ID for the product, e.g. `ActionaNG.Actiona` (or `ActionaNG.Actiona.Portable` for a separate channel).
- Register it properly (shortcut + AUMID for unpackaged desktop toast scenarios).

For multiple concurrent Actiona instances from different paths:

- Do **not** create a new AUMID per process instance.
- AUMID is app identity, not instance identity; for the same app experience, reuse the same AUMID across processes.
- Use per-instance `tag/group/launch args` to disambiguate instance behavior.
- Only use multiple AUMIDs if they are intentionally different app experiences/channels (stable/dev/portable), not per run or per temp SFX extraction.

Sources:
- `winrt_toast_reborn` `ToastManager` docs (PowerShell AUMID, marked experimental): https://docs.rs/winrt-toast-reborn/latest/winrt_toast_reborn/struct.ToastManager.html
- `winrt_toast_reborn` `register` docs: https://docs.rs/winrt-toast-reborn/latest/winrt_toast_reborn/fn.register.html
- Microsoft: AppUserModelID guidance (same explicit ID across related windows/processes/shortcuts): https://learn.microsoft.com/en-us/windows/win32/shell/appids
- Microsoft: desktop toast + shortcut/AUMID requirement: https://learn.microsoft.com/en-us/windows/win32/shell/enable-desktop-toast-with-appusermodelid
- Microsoft: unpackaged desktop toast activation notes: https://learn.microsoft.com/en-us/windows/apps/design/shell/tiles-and-notifications/toast-desktop-apps

use winrt_toast_reborn::{Action, Toast, ToastManager};
use winrt_toast_reborn::url;

let manager = ToastManager::new("ActionaNG.Actiona");
let instance_id = "inst-42";
let run_id = "run-9001";

let launch_args = url::form_urlencoded::Serializer::new(String::new())
    .append_pair("kind", "toast_click")
    .append_pair("instance", instance_id)
    .append_pair("run", run_id)
    .finish();

let mut toast = Toast::new();
toast
    .tag(format!("run:{run_id}"))          // primary key for one toast
    .group(format!("instance:{instance_id}")) // group key for many toasts
    .launch(launch_args)                   // args for toast-body activation
    .action(Action::new("Open", "action=open", "")); // button args in `action.arg`

manager
    .on_activated(None, |activated| {
        if let Some(a) = activated {
            // `a.arg` is the activation argument string
            // `a.tag` contains toast tag if set
            println!("arg={}, tag={:?}", a.arg, a.tag);
        } else {
            // crate sample handles this case too
            println!("toast activated without action payload");
        }
    })
    .show(&toast)?;
