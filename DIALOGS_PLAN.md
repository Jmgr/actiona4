# Cancellable dialogs: context and remaining work

## Why this work exists

The goal is a hot-reloading mode for `actiona-run run`: when the script file changes, the
running script is stopped and the new version started.

### Design chosen for hot reloading

- **A supervisor process.** `actiona-run --watch script.ts` becomes a thin supervisor that never
  runs scripts. It owns the file watcher, the terminal and Ctrl+C, the tray icon and the stop
  hotkey. It starts the script as a child, `actiona-run run script.ts --no-tray --no-stop-hotkey`
  plus a hidden "supervised" flag, and restarts it on every change.
  - Rejected: reloading inside one process. That needs every root-token use in `core` audited and
    the runtime split into process-level and script-level state.
  - Rejected: the process starting a new copy of itself and exiting. On Unix that hands the
    terminal back to the shell, and it can't exit cleanly while a dialog is open.
- **Stopping the child:**
  1. Graceful stop request: `SIGTERM` on Unix, an IPC message on Windows. The child cancels its
     root token and releases held keys and mouse buttons straight away, before waiting for the
     script.
  2. After a grace period, the child cleans up and exits by itself (`std::process::exit`).
  3. Last resort only: the supervisor kills the child's whole process group (Linux) or job
     object (Windows). Detached processes started by scripts then need their own process group,
     or to break away from the job, to survive.
- **Script API:** `exit()` ends the current run, so the supervisor keeps watching. A separate
  call, such as `exit({ quit: true })` or `app.quit()`, would quit the whole process.
- **Leftover side effects found while investigating:**
  - `enigo` releases held keys only when it is dropped.
  - Mouse buttons pressed with `mouse.press()` are never released. `Mouse::pressed_buttons` tracks
    them, but nothing uses it on shutdown.
  - Macro playback releases its own input; processes started with handles are killed on drop;
    audio and the selection overlay listen to the root token.
- **Blocker:** a script waiting on a dialog could not be stopped. `rfd` and `rustydialogs` give
  no way to close an open dialog, and some of their dialogs block a `spawn_blocking` thread. This
  also breaks the stop hotkey and Ctrl+C outside watch mode.

### Why a new crate

No existing crate covers our dialogs with cancellation. Checked: `rfd`, `rustydialogs`,
`native-dialog`, `dialog`, `zenity-rs`, `xdialog`, `win-task-dialog`. So `crates/dialogs` was
written.

**Its rule:** every dialog is an `async fn`, and dropping its future closes the dialog. Nothing
takes a token: callers use `cancel_on` (`crates/core/src/lib.rs`), `tokio::select!` or
`tokio::time::timeout`.

## State of `crates/dialogs` (phases 1 to 4 done)

**Public API** (`src/lib.rs`, `src/options.rs`, `src/progress.rs`):
- `Dialogs::new()`, plus `Dialogs::with_backends(LinuxBackends)` on Linux.
- Dialogs: `message_box`, `text_input`, `color_picker`, `pick_file`, `pick_files`,
  `pick_folder`, `pick_folders`, `save_file`, `select_one`, `select_many`, `date`, `progress`.
- **`Progress` handle:**
  - `set_value(Option<f64>)`, where `None` shows a busy bar, and `set_text`. Only the latest
    state is shown, so updates are cheap.
  - `is_cancelled()` and `cancelled().await` report the user cancelling or closing the dialog.
  - `close().await` waits until the dialog is gone; dropping the handle also closes it.
- **Errors:** `NoBackend`, `Unsupported`, `InvalidOptions`, `Backend`, `Io`, `DBus` (Unix),
  `Windows` (Windows).

### Linux (`src/linux/`): done and verified

**Backends:**
- `zenity` or `kdialog`, whichever is installed (`kdialog` preferred on KDE), run as child
  processes with `kill_on_drop`.
- The tool is chosen through the `Tool` trait with `static_dispatch` on `LinuxTool`.
- File dialogs use the xdg-desktop-portal file chooser over `zbus` (`linux/portal.rs`). Dropping
  the future calls the portal request's `Close()`. Without a portal, they fall back to the tool.
- `ACTIONA_DIALOGS_BACKEND=zenity|kdialog|portal` forces a backend.

**Quirks handled, each found by reading the tools' source code or by screenshots:**
- **Arguments:** `zenity` values use the `--opt=value` form. `kdialog` values that could start
  with `-` go after `--`.
- **`zenity` escaping:**
  - It unescapes C escape sequences in the prompts of entry, list, calendar and progress dialogs.
  - It reads list, calendar and progress prompts as Pango markup.
  - Entry prompts treat `_` as a keyboard-shortcut marker.
  - Message boxes get `--no-markup`.
- **`zenity` list items:** an item starting with `-` is dropped even after `--`, so list rows are
  sent on standard input.
- **`zenity` version differences:**
  - Progress text updates are markup from 4.0.3 on, and plain text before that.
  - From 4.0 up to before 4.1.99, `--filename=DIR/` selects the folder inside its parent and
    drops the filters. A missing file inside the folder (`actiona-missing-file`) is used instead
    for those versions.
  - The same versions can't pre-fill a save name for a file that doesn't exist yet.
  - The initial `--percentage` must be a whole number.
- **`kdialog` texts:**
  - It unescapes `\n` and `\\` in prompts, so backslashes are doubled.
  - Qt labels guess HTML (`Qt::mightBeRichText`), so a zero-width space is inserted after each
    `<`.
  - The single-line input's prompt treats `&` as a keyboard-shortcut marker.
- **`kdialog` options:** the initial colour goes through `--default`. Selecting several folders
  is not supported.
- **`kdialog` progress:** `kdialog --progressbar` starts a detached helper and exits, so
  `kdialog_progress_helper` is started directly as our own child process and driven over D-Bus.

**Verified:**
- 41 unit tests.
- Real dialogs under Xvfb, on a private D-Bus with service activation disabled, so nothing
  appears on the desktop. Screenshots were checked, drop tests pass with both tools, and no
  processes were left behind.
- The portal has **not** been seen working: it can't start inside a private bus.

### Windows (`src/windows/`): written, compiles, never run

**Implementation:**
- Each dialog runs on a dedicated `std::thread` (`windows/thread.rs`). This is deliberate rather
  than `spawn_blocking`:
  - A runtime shutdown waits for blocking tasks.
  - Pool threads would keep message-queue and COM state, including a stray `WM_QUIT`.
- **Closing on drop:** `WM_QUIT` is posted to the dialog's thread and `WM_CLOSE` to its windows.
  A lock prevents posting once the thread has finished, since its ID could be reused.
- **Message box and progress:** task dialogs (`windows/task_dialog.rs`).
  - A timer callback every 200 ms closes them, using `TDM_CLICK_BUTTON` with `IDCANCEL`.
  - It also applies progress updates.
  - `TaskDialogIndirect` is looked up at runtime, so an executable without the Common Controls 6
    manifest gets an error instead of failing to start.
  - When any button is relabelled, all of them become custom buttons; the others default to
    English.
- **Colour picker:** `ChooseColorW`, with the title set from its hook (`windows/color.rs`).
- **File dialogs:** `IFileOpenDialog`/`IFileSaveDialog` (`windows/file.rs`).
- `crates/dialogs/build.rs` embeds `assets/windows-manifest.xml` into the crate's own test
  binaries, like `core` does.

**To check on Windows** with `cargo make dialogs-manual` (backend name `windows`):
- Each kind of dialog closes when dropped. The file dialog is the least certain.
- Closing an OK-only message box returns `Ok`.
- Progress switches between busy and counting, and Cancel is reported.
- A progress dialog that can't be cancelled closes when dropped. No manual test covers it yet.
- The colour picker's custom title.

The CI `windows-test` job runs the Windows-only unit tests.

### Tooling

- `cargo make dialogs-manual` runs the interactive tests in `crates/dialogs/tests/manual.rs`.
  `ACTIONA_DIALOGS_BACKEND` limits them to one backend.
- **Headless testing on Linux:**
  - Run `xvfb-run -a dbus-run-session --config-file=<config without service activation> -- …`.
  - The private bus must be started *inside* `xvfb-run`, otherwise the services it starts draw
    on the real display.
  - Use `pgrep -f '[p]attern'` with the bracket trick, or `pgrep -f` matches the shell running it.

## Phase 5: the remaining Windows dialogs

Windows has no ready-made dialogs for text input, selection or dates, so these are small windows
of our own, built from standard controls so they look native.

1. **A shared window helper** (`windows/window.rs`):
   - A dialog window with a prompt label, one hosted control, and OK and Cancel buttons.
   - Its own message loop, which ends on `WM_QUIT`. `thread::run`'s closing therefore works
     unchanged, and `WM_CLOSE` cancels the dialog.
   - Enter accepts, Escape cancels.
   - DPI-aware sizing; the manifest already declares per-monitor v2 awareness.
2. **Text input** (`windows/input.rs`):
   - A single-line edit, `ES_PASSWORD` for passwords, or `ES_MULTILINE | ES_WANTRETURN` for
     multi-line text.
   - `rustydialogs`' `win32/input.rs` (MIT) is a starting point.
3. **Select one and select many** (`windows/select.rs`):
   - A list view, with `LVS_EX_CHECKBOXES` for multiple selection.
   - Single selection starts with the first item selected, or the first index in `selected`.
   - Returns indices.
4. **Date** (`windows/date.rs`):
   - The month calendar control (`MONTHCAL_CLASS`): `MCM_SETCURSEL` to set the date,
     `MCM_GETCURSEL` to read it, converted to `jiff::civil::Date`.
5. **API:** remove `#[cfg(unix)]` from `text_input`, `select_one`, `select_many` and `date` in
   `lib.rs`, add the matching methods to `windows::Backends`, and remove the `cfg(unix)`
   attributes in `tests/manual.rs`.
6. Run `cargo make lint` and `cargo make lint-windows`. Then test on Windows: rendering,
   keyboard handling, and closing when dropped.

## Phase 6: switching `core` to the crate

1. **Dependency:** add `dialogs` to `crates/core/Cargo.toml`, and keep one `Dialogs` in the
   `JsDialogs` singleton (`crates/core/src/api/dialogs/js.rs`).
2. **Replace the implementations** in `crates/core/src/api/dialogs/{mod.rs, file_dialog.rs,
   native_dialog.rs}` with calls to the crate, converting the option types. Remove `rfd` and
   `rustydialogs` from `core`.
   - `rfd` stays for the two synchronous dialogs in `crates/common/src/sentry.rs` and
     `crates/run/src/lib.rs`.
3. **Cancellation:** wrap each JS dialog method in `task_with_token`
   (`crates/core/src/api/js/task.rs`), as `keyboard.waitForKeys` does
   (`crates/core/src/api/keyboard/js.rs`). Dialogs then:
   - accept a `signal` option;
   - close when the script is stopped;
   - get a `timeout` option, implemented with `tokio::time::timeout`.
   - **To decide:** what a timed-out dialog returns. One option is a JS-side
     `MessageBoxResult.Timeout`, and `null` for the others.
4. **Message box labels:** replace `MessageBoxButtons`' custom variants (`OkCustom`,
   `OkCancelCustom`, `YesNoCancelCustom`) with a `labels` option. This is a breaking change; the
   old forms could stay as deprecated aliases.
5. **New JS API:** `dialogs.selectOne`, `dialogs.selectMany`, `dialogs.date` and
   `dialogs.progress`. `progress` returns a `Progress` host class with:
   - `value`: a number, or `null` for a busy bar;
   - `text`;
   - `cancelled`, plus a way to wait for it;
   - `close()`.
6. **Decide on `TaskTracker`.** Optionally add `Dialogs::with_task_tracker(tracker)`, so that
   `run_impl`'s `task_tracker.wait()` also waits until dialogs are gone. The Windows threads
   would signal through a oneshot channel awaited by a tracked task; Linux progress tasks would
   use `tracker.spawn`. The trade-off: a dialog that fails to close would hang shutdown.
7. **Documentation:** update the Rust doc comments, then run `cargo make doc` to regenerate
   `crates/run/assets/index.d.ts`. Add end-to-end tests where they can run without a person
   clicking.
8. **Windows test binaries:** with `TaskDialogIndirect` looked up at runtime, binaries without
   the manifest can still start, but their message boxes and progress dialogs fail with an error.
   Check that every binary that shows dialogs embeds the manifest: `run`, `core`'s tests, and any
   others.

## After phase 6

- **The `--watch` supervisor in `run`:**
  - A file watcher (`notify` with debouncing; watch the parent folder, since editors save by
    renaming).
  - The supervised child, graceful stop, self-exit fallback, and process group or job object.
  - Releasing held keys and mouse buttons on stop.
  - The `exit()` / quit semantics.
- **Hot reloading inside one process stays rejected** (see above).
