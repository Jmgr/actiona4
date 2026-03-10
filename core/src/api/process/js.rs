use std::{collections::HashMap, sync::Arc};

use derive_more::Display;
use macros::{FromJsObject, FromSerde, IntoSerde, js_class, js_methods, options, platform};
use rquickjs::{
    Ctx, Exception, Function, JsLifetime, Object, Promise, Result, Value, atom::PredefinedAtom,
    class::Trace, prelude::Opt,
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;

#[cfg(unix)]
use crate::api::system::processes::Signal;
use crate::{
    IntoJsResult,
    api::{
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_enum, register_host_class},
            task::task_with_token,
        },
        process::{ProcessRunner, ShellOptions, StartProcessOptions},
    },
    cancel_on,
    runtime::WithUserData,
    types::{
        display::{DisplayFields, display_with_type},
        pid::Pid,
    },
};

/// Options for starting a process.
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsStartProcessOptions {
    /// Arguments to pass to the command.
    pub args: Vec<String>,

    /// Working directory for the process.
    pub working_directory: Option<String>,

    /// Environment variables for the process.
    pub env: Option<HashMap<String, String>>,

    /// Abort signal to kill the process.
    pub signal: Option<JsAbortSignal>,
}

impl JsStartProcessOptions {
    fn into_inner(self) -> StartProcessOptions {
        StartProcessOptions {
            args: self.args,
            working_directory: self.working_directory,
            env: self.env,
        }
    }
}

/// Options for running a shell command.
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsShellOptions {
    /// Shell to use. On Linux defaults to `$SHELL` (or `bash` if unset).
    /// On Windows defaults to `powershell`.
    pub shell: Option<String>,

    /// Abort signal to cancel the operation.
    pub signal: Option<JsAbortSignal>,
}

/// Unix signal.
///
/// ```ts
/// await process.sendSignal(1234, Signal.Term);
/// ```
///
#[platform(only = "linux")]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    Hash,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "Signal")]
pub enum JsSignal {
    /// `SIGHUP` - hang up; often used to request config reload.
    /// `Signal.Hup`
    Hup,

    /// `SIGINT` - interrupt (like Ctrl-C).
    /// `Signal.Int`
    Int,

    /// `SIGQUIT` - quit; similar to `SIGINT`, often with core dump.
    /// `Signal.Quit`
    Quit,

    /// `SIGTERM` - polite termination request.
    /// `Signal.Term`
    Term,

    /// `SIGKILL` - force kill immediately.
    /// `Signal.Kill`
    Kill,

    /// `SIGSTOP` - stop/suspend execution immediately.
    /// `Signal.Stop`
    Stop,

    /// `SIGTSTP` - terminal stop (like Ctrl-Z).
    /// `Signal.Tstp`
    Tstp,

    /// `SIGCONT` - continue a stopped process.
    /// `Signal.Cont`
    Cont,

    /// `SIGTTIN` - background process attempted terminal input.
    /// `Signal.Ttin`
    Ttin,

    /// `SIGTTOU` - background process attempted terminal output.
    /// `Signal.Ttou`
    Ttou,

    /// `SIGWINCH` - terminal window size changed.
    /// `Signal.Winch`
    Winch,

    /// `SIGUSR1` - user-defined signal 1.
    /// `Signal.Usr1`
    Usr1,

    /// `SIGUSR2` - user-defined signal 2.
    /// `Signal.Usr2`
    Usr2,
}

#[cfg(unix)]
impl From<JsSignal> for Signal {
    fn from(value: JsSignal) -> Self {
        match value {
            JsSignal::Hup => Self::Hup,
            JsSignal::Int => Self::Int,
            JsSignal::Quit => Self::Quit,
            JsSignal::Term => Self::Term,
            JsSignal::Kill => Self::Kill,
            JsSignal::Stop => Self::Stop,
            JsSignal::Tstp => Self::Tstp,
            JsSignal::Cont => Self::Cont,
            JsSignal::Ttin => Self::Ttin,
            JsSignal::Ttou => Self::Ttou,
            JsSignal::Winch => Self::Winch,
            JsSignal::Usr1 => Self::Usr1,
            JsSignal::Usr2 => Self::Usr2,
        }
    }
}

/// Start and manage child processes.
///
/// ```ts
/// const handle = process.start("echo", { args: ["hello world"] });
/// for await (const line of handle.stdout) {
///     println(line);
/// }
/// const result = await handle.closed;
/// println(result.exitCode);
/// ```
///
/// ```ts
/// const result = await process.startAndWait("ls", { args: ["-la"] });
/// println(result.stdout);
/// ```
///
/// ```ts
/// const pid = process.startDetached("my-server", { args: ["--port", "8080"] });
/// println(pid);
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsProcess {
    inner: ProcessRunner,
}

impl<'js> Trace<'js> for JsProcess {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsProcess {
    fn register_dependencies(ctx: &Ctx<'js>) -> Result<()> {
        register_host_class::<JsProcessHandle>(ctx)?;
        register_host_class::<JsProcessExitResult>(ctx)?;
        register_enum::<JsSignal>(ctx)?;
        Ok(())
    }
}

impl JsProcess {
    /// @skip
    #[must_use]
    pub const fn new(task_tracker: tokio_util::task::TaskTracker) -> Self {
        Self {
            inner: ProcessRunner::new(task_tracker),
        }
    }
}

#[js_methods]
impl JsProcess {
    /// Starts a process and returns a `ProcessHandle` for interacting with it.
    ///
    /// ```ts
    /// const handle = process.start("echo", { args: ["hello world"] });
    /// for await (const line of handle.stdout) {
    ///     println(line);
    /// }
    /// const result = await handle.closed;
    /// println(result.exitCode);
    /// ```
    ///
    /// ```ts
    /// const handle = process.start("cat");
    /// await handle.write("hello\n");
    /// await handle.closeStdin();
    /// for await (const line of handle.stdout) {
    ///     println(line);
    /// }
    /// await handle.closed;
    /// ```
    pub fn start<'js>(
        &self,
        ctx: Ctx<'js>,
        command: String,
        options: Opt<JsStartProcessOptions>,
    ) -> Result<JsProcessHandle> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let start_options = options.into_inner();

        let cancellation_token = signal.map_or_else(
            || ctx.user_data().child_cancellation_token(),
            |s| s.into_token(),
        );

        let started = self
            .inner
            .start(&command, start_options, cancellation_token.clone())
            .into_js_result(&ctx)?;

        Ok(JsProcessHandle {
            pid: started.pid.into(),
            stdin: started.stdin.map(|s| Arc::new(Mutex::new(Some(s)))),
            stdout_receiver: started.stdout_receiver.map(|r| Arc::new(Mutex::new(r))),
            stderr_receiver: started.stderr_receiver.map(|r| Arc::new(Mutex::new(r))),
            child: Arc::new(Mutex::new(started.child)),
            cancellation_token,
        })
    }

    /// Starts a process, waits for it to finish, and returns the exit result
    /// including captured stdout and stderr.
    ///
    /// ```ts
    /// const result = await process.startAndWait("ls", { args: ["-la"] });
    /// println(result.stdout);
    /// println(result.exitCode);
    /// ```
    ///
    /// @returns Task<ProcessExitResult>
    pub fn start_and_wait<'js>(
        &self,
        ctx: Ctx<'js>,
        command: String,
        options: Opt<JsStartProcessOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let start_options = options.into_inner();
        let inner = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, cancel_token| {
            let process_result = inner
                .start_and_wait(&command, start_options, cancel_token)
                .await
                .into_js_result(&ctx)?;
            let (exit_code, stdout, stderr) = process_result.into_parts();

            Ok(JsProcessExitResult {
                pid: None,
                exit_code,
                stdout: Some(stdout),
                stderr: Some(stderr),
            })
        })
    }

    /// Starts a detached process and returns its PID.
    /// The process will continue running after the script exits.
    ///
    /// ```ts
    /// const pid = process.startDetached("my-server", { args: ["--port", "8080"] });
    /// println(`Started server with PID: ${pid}`);
    /// ```
    pub fn start_detached(
        &self,
        ctx: Ctx<'_>,
        command: String,
        options: Opt<JsStartProcessOptions>,
    ) -> Result<u32> {
        let options = options.0.unwrap_or_default();
        let start_options = options.into_inner();

        let pid = self
            .inner
            .start_detached(&command, start_options)
            .into_js_result(&ctx)?;

        Ok(pid.into())
    }

    /// Runs a command through the system shell, similar to C's `system()` function.
    ///
    /// Stdio is inherited from the current process: if a console window is open the
    /// command runs inside it; otherwise the OS opens a new console window for it.
    ///
    /// The default shell is platform-specific:
    /// - **Linux** – the value of `$SHELL`, falling back to `bash`.
    /// - **Windows** – `powershell`.
    ///
    /// A custom shell can be supplied via `options.shell`. On Windows the command
    /// flag (`/C`, `-Command`, or `-c`) is inferred automatically from the shell name.
    ///
    /// ```ts
    /// // Clear the screen (works on Windows with cmd/powershell and on Unix)
    /// await process.shell("cls");
    /// ```
    ///
    /// ```ts
    /// // Use a specific shell
    /// await process.shell("echo hello", { shell: "zsh" });
    /// ```
    ///
    /// @returns Task<number | undefined>
    pub fn shell<'js>(
        &self,
        ctx: Ctx<'js>,
        command: String,
        options: Opt<JsShellOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let shell_options = ShellOptions {
            shell: options.shell,
        };
        let inner = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, cancel_token| {
            inner
                .shell(&command, shell_options, cancel_token)
                .await
                .into_js_result(&ctx)
        })
    }

    /// Kill a process by PID (SIGKILL on Unix, TerminateProcess on Windows).
    ///
    /// ```ts
    /// process.kill(1234);
    /// ```
    pub fn kill(&self, ctx: Ctx<'_>, pid: u32) -> Result<()> {
        let pid = Pid::try_from(pid).into_js_result(&ctx)?;
        super::kill_by_pid(pid).into_js_result(&ctx)
    }

    /// Gracefully terminate a process by PID (SIGTERM on Unix, WM_CLOSE on Windows).
    ///
    /// ```ts
    /// process.terminate(1234);
    /// ```
    pub fn terminate(&self, ctx: Ctx<'_>, pid: u32) -> Result<()> {
        let pid = Pid::try_from(pid).into_js_result(&ctx)?;
        super::terminate_by_pid(pid).into_js_result(&ctx)
    }

    /// Send a signal to a process by PID.
    ///
    /// ```ts
    /// process.sendSignal(1234, Signal.Term);
    /// ```
    ///
    #[platform(only = "linux")]
    pub fn send_signal(&self, ctx: Ctx<'_>, pid: u32, signal: JsSignal) -> Result<()> {
        #[cfg(unix)]
        {
            let pid = Pid::try_from(pid).into_js_result(&ctx)?;
            super::send_signal(pid, signal.into()).into_js_result(&ctx)
        }

        #[cfg(not(unix))]
        {
            let _ = (pid, signal);
            Err(Exception::throw_message(
                &ctx,
                "process.sendSignal is only supported on Unix platforms",
            ))
        }
    }

    /// Returns a string representation of the `process` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Process", &self.inner)
    }
}

/// A handle to a running process.
///
/// Provides access to the process's PID, stdin, stdout, stderr, and allows
/// waiting for the process to exit or killing it.
///
/// ```ts
/// const handle = process.start("echo", { args: ["hello"] });
/// for await (const line of handle.stdout) {
///     println(line);
/// }
/// const result = await handle.closed;
/// println(result.exitCode);
/// ```
#[derive(JsLifetime)]
#[js_class]
pub struct JsProcessHandle {
    pid: u32,
    stdin: Option<Arc<Mutex<Option<tokio::process::ChildStdin>>>>,
    stdout_receiver: Option<Arc<Mutex<mpsc::Receiver<String>>>>,
    stderr_receiver: Option<Arc<Mutex<mpsc::Receiver<String>>>>,
    child: Arc<Mutex<tokio::process::Child>>,
    cancellation_token: CancellationToken,
}

impl<'js> HostClass<'js> for JsProcessHandle {}

impl<'js> Trace<'js> for JsProcessHandle {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

/// Creates an async iterable object from an mpsc receiver.
/// The returned object has `[Symbol.asyncIterator]()` which returns an iterator with `next()`.
/// A tokio Mutex is required here because `next()` calls `rx.recv().await` while holding the lock.
fn make_async_iterable<'js>(
    ctx: &Ctx<'js>,
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
    cancellation_token: CancellationToken,
) -> Result<Object<'js>> {
    let obj = Object::new(ctx.clone())?;
    let iter_fn = Function::new(ctx.clone(), move |ctx: Ctx<'js>| {
        let iter = Object::new(ctx.clone())?;
        let rx = rx.clone();
        let token = cancellation_token.clone();

        let next_fn = Function::new(ctx, move |ctx: Ctx<'js>| {
            let rx = rx.clone();
            let token = token.clone();
            let local_ctx = ctx.clone();

            let promise = Promise::wrap_future(&local_ctx, async move {
                let value = {
                    let mut rx = rx.lock().await;

                    cancel_on(&token, rx.recv()).await.unwrap_or(None)
                };

                let result = Object::new(ctx.clone())?;
                match value {
                    Some(line) => {
                        result.set("value", line)?;
                        result.set("done", false)?;
                    }
                    None => {
                        result.set("done", true)?;
                    }
                }
                Ok::<Value<'js>, rquickjs::Error>(result.into_value())
            })?;

            Ok::<Promise<'js>, rquickjs::Error>(promise)
        })?;

        iter.set("next", next_fn)?;
        Ok::<Object<'js>, rquickjs::Error>(iter)
    })?;
    obj.set(PredefinedAtom::SymbolAsyncIterator, iter_fn)?;

    Ok(obj)
}

#[js_methods]
impl JsProcessHandle {
    /// Process ID.
    #[get]
    #[must_use]
    pub const fn pid(&self) -> u32 {
        self.pid
    }

    /// An async iterator that yields lines from the process's standard output.
    ///
    /// ```ts
    /// const handle = process.start("echo", { args: ["hello"] });
    /// for await (const line of handle.stdout) {
    ///     println(line);
    /// }
    /// ```
    ///
    /// @returns AsyncIterableIterator<string>
    #[get]
    pub fn stdout<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let Some(rx) = &self.stdout_receiver else {
            return Err(Exception::throw_message(&ctx, "stdout is not available"));
        };

        make_async_iterable(&ctx, rx.clone(), self.cancellation_token.clone())
    }

    /// An async iterator that yields lines from the process's standard error.
    ///
    /// ```ts
    /// const handle = process.start("my-command");
    /// for await (const line of handle.stderr) {
    ///     println(`error: ${line}`);
    /// }
    /// ```
    ///
    /// @returns AsyncIterableIterator<string>
    #[get]
    pub fn stderr<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        let Some(rx) = &self.stderr_receiver else {
            return Err(Exception::throw_message(&ctx, "stderr is not available"));
        };

        make_async_iterable(&ctx, rx.clone(), self.cancellation_token.clone())
    }

    /// Write data to the process's stdin.
    ///
    /// ```ts
    /// const handle = process.start("cat");
    /// await handle.write("hello\n");
    /// ```
    pub async fn write(&self, ctx: Ctx<'_>, data: String) -> Result<()> {
        let stdin_holder = self
            .stdin
            .as_ref()
            .ok_or_else(|| Exception::throw_message(&ctx, "stdin is not available"))?;

        let mut guard = stdin_holder.lock().await;
        let stdin = guard
            .as_mut()
            .ok_or_else(|| Exception::throw_message(&ctx, "stdin has been closed"))?;

        tokio::io::AsyncWriteExt::write_all(stdin, data.as_bytes())
            .await
            .into_js_result(&ctx)?;
        tokio::io::AsyncWriteExt::flush(stdin)
            .await
            .into_js_result(&ctx)?;
        Ok(())
    }

    /// Close the process's stdin. This signals EOF to the child process,
    /// which is necessary for programs that read until EOF (like `cat`).
    ///
    /// ```ts
    /// const handle = process.start("cat");
    /// await handle.write("hello\n");
    /// await handle.closeStdin();
    /// ```
    pub async fn close_stdin(&self) {
        if let Some(stdin_holder) = &self.stdin {
            let mut guard = stdin_holder.lock().await;
            guard.take();
        }
    }

    /// Kill the process immediately (SIGKILL on Unix, TerminateProcess on Windows).
    ///
    /// ```ts
    /// const handle = process.start("sleep", { args: ["100"] });
    /// handle.kill();
    /// ```
    pub fn kill(&self, ctx: Ctx<'_>) -> Result<()> {
        let pid = Pid::try_from(self.pid).into_js_result(&ctx)?;
        super::kill_by_pid(pid).into_js_result(&ctx)
    }

    /// Gracefully terminate the process (SIGTERM on Unix, WM_CLOSE on Windows).
    ///
    /// ```ts
    /// const handle = process.start("sleep", { args: ["100"] });
    /// handle.terminate();
    /// ```
    pub fn terminate(&self, ctx: Ctx<'_>) -> Result<()> {
        let pid = Pid::try_from(self.pid).into_js_result(&ctx)?;
        super::terminate_by_pid(pid).into_js_result(&ctx)
    }

    /// A promise that resolves with the exit result when the process closes.
    ///
    /// ```ts
    /// const handle = process.start("ls");
    /// const result = await handle.closed;
    /// println(result.exitCode);
    /// ```
    ///
    /// @returns Task<ProcessExitResult>
    #[get]
    pub fn closed<'js>(&self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
        let child = self.child.clone();
        let token = self.cancellation_token.clone();
        let pid = self.pid;

        task_with_token(ctx, token, async move |ctx, cancel_token| {
            let mut child = child.lock().await;

            let status = tokio::select! {
                _ = cancel_token.cancelled() => {
                    child.kill().await.into_js_result(&ctx)?;
                    child.wait().await.into_js_result(&ctx)?
                },
                status = child.wait() => {
                    status.into_js_result(&ctx)?
                },
            };

            Ok(JsProcessExitResult {
                pid: Some(pid),
                exit_code: status.code(),
                stdout: None,
                stderr: None,
            })
        })
    }

    /// Returns a string representation of this process handle.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "ProcessHandle",
            DisplayFields::default()
                .display("pid", self.pid)
                .finish_as_string(),
        )
    }
}

/// The result of a process that has finished.
///
/// ```ts
/// const handle = process.start("ls");
/// const result = await handle.closed;
/// if (result.exitCode === 0) {
///     println("success");
/// }
/// ```
///
/// ```ts
/// const result = await process.startAndWait("echo", { args: ["hello"] });
/// println(result.stdout);
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsProcessExitResult {
    pid: Option<u32>,
    exit_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
}

impl<'js> HostClass<'js> for JsProcessExitResult {}

impl<'js> Trace<'js> for JsProcessExitResult {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[js_methods]
impl JsProcessExitResult {
    /// The process ID. Only available when using `handle.closed`.
    #[get]
    #[must_use]
    pub const fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// The exit code of the process. `undefined` if the process was killed by a signal.
    #[get]
    #[must_use]
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// The captured stdout output. Only available when using `startAndWait`.
    #[get]
    #[must_use]
    pub fn stdout(&self) -> Option<&str> {
        self.stdout.as_deref()
    }

    /// The captured stderr output. Only available when using `startAndWait`.
    #[get]
    #[must_use]
    pub fn stderr(&self) -> Option<&str> {
        self.stderr.as_deref()
    }

    /// Returns a string representation of this process exit result.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "ProcessExitResult",
            DisplayFields::default()
                .display_if_some("pid", &self.pid)
                .display_if_some("exitCode", &self.exit_code)
                .finish_as_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use std::time::Duration;

    #[cfg(windows)]
    use tokio::time::timeout;

    use crate::runtime::Runtime;

    /// Returns a shell command string for running a shell expression.
    /// On Unix: `sh -c "<expr>"`. On Windows: `cmd /c <expr>`.
    #[cfg(unix)]
    fn shell_cmd(expr: &str) -> String {
        format!(
            r#"process.start("sh", {{ args: ["-c", {expr}] }})"#,
            expr = serde_json::to_string(expr).unwrap()
        )
    }

    #[cfg(windows)]
    fn shell_cmd(expr: &str) -> String {
        format!(
            r#"process.start("cmd", {{ args: ["/c", {expr}] }})"#,
            expr = serde_json::to_string(expr).unwrap()
        )
    }

    #[cfg(unix)]
    fn shell_cmd_and_wait(expr: &str) -> String {
        format!(
            r#"await process.startAndWait("sh", {{ args: ["-c", {expr}] }})"#,
            expr = serde_json::to_string(expr).unwrap()
        )
    }

    #[cfg(windows)]
    fn shell_cmd_and_wait(expr: &str) -> String {
        format!(
            r#"await process.startAndWait("cmd", {{ args: ["/c", {expr}] }})"#,
            expr = serde_json::to_string(expr).unwrap()
        )
    }

    /// Returns a long-running command suitable for testing kill/terminate.
    #[cfg(unix)]
    const LONG_SLEEP: &str = r#"process.start("sleep", { args: ["100"] })"#;

    #[cfg(windows)]
    const LONG_SLEEP: &str = r#"process.start("ping", { args: ["-n", "100", "127.0.0.1"] })"#;

    /// Returns a short detached command.
    #[cfg(unix)]
    const SHORT_DETACHED: &str = r#"process.startDetached("sleep", { args: ["0.1"] })"#;

    #[cfg(windows)]
    const SHORT_DETACHED: &str =
        r#"process.startDetached("ping", { args: ["-n", "1", "127.0.0.1"] })"#;

    #[test]
    fn test_start_echo() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cmd = shell_cmd("echo hello world");
            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                const handle = {cmd};
                let output = "";
                for await (const line of handle.stdout) {{
                    output += line;
                }}
                await handle.closed;
                output
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, "hello world");
        });
    }

    #[test]
    fn test_start_cat_stdin() {
        Runtime::test_with_script_engine(async |script_engine| {
            #[cfg(unix)]
            let cmd = r#"process.start("cat")"#;
            #[cfg(windows)]
            let cmd = r#"process.start("findstr", { args: [".*"] })"#;

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                const handle = {cmd};
                await handle.write("test input\n");
                await handle.closeStdin();
                let output = "";
                for await (const line of handle.stdout) {{
                    output += line;
                }}
                await handle.closed;
                output
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, "test input");
        });
    }

    #[test]
    fn test_exit_code() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cmd = shell_cmd("exit 42");
            let exit_code = script_engine
                .eval_async::<i32>(&format!(
                    r#"
                const handle = {cmd};
                const result = await handle.closed;
                result.exitCode
                "#
                ))
                .await
                .unwrap();

            assert_eq!(exit_code, 42);
        });
    }

    #[test]
    fn test_stderr() {
        Runtime::test_with_script_engine(async |script_engine| {
            #[cfg(unix)]
            let expr = "echo error >&2";
            #[cfg(windows)]
            let expr = "echo error 1>&2";

            let cmd = shell_cmd(expr);
            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                const handle = {cmd};
                let output = "";
                for await (const line of handle.stderr) {{
                    output += line;
                }}
                await handle.closed;
                output
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result.trim(), "error");
        });
    }

    #[test]
    fn test_kill() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                const handle = {LONG_SLEEP};
                handle.kill();
                await handle.closed;
                "#
                ))
                .await
                .unwrap();
        });
    }

    #[test]
    fn test_start_detached() {
        Runtime::test_with_script_engine(async |script_engine| {
            let pid = script_engine
                .eval_async::<u32>(&format!(
                    r#"
                const pid = {SHORT_DETACHED};
                pid
                "#
                ))
                .await
                .unwrap();

            assert!(pid > 0);
        });
    }

    #[test]
    fn test_pid() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cmd = shell_cmd("echo test");
            let pid = script_engine
                .eval_async::<u32>(&format!(
                    r#"
                const handle = {cmd};
                const pid = handle.pid;
                await handle.closed;
                pid
                "#
                ))
                .await
                .unwrap();

            assert!(pid > 0);
        });
    }

    #[test]
    fn test_start_and_wait() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cmd = shell_cmd_and_wait("echo hello world");
            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                const result = {cmd};
                result.stdout
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result, "hello world");
        });
    }

    #[test]
    fn test_start_and_wait_exit_code() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cmd = shell_cmd_and_wait("exit 7");
            let exit_code = script_engine
                .eval_async::<i32>(&format!(
                    r#"
                const result = {cmd};
                result.exitCode
                "#
                ))
                .await
                .unwrap();

            assert_eq!(exit_code, 7);
        });
    }

    #[test]
    fn test_start_and_wait_stderr() {
        Runtime::test_with_script_engine(async |script_engine| {
            #[cfg(unix)]
            let expr = "echo err >&2";
            #[cfg(windows)]
            let expr = "echo err 1>&2";

            let cmd = shell_cmd_and_wait(expr);
            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                const result = {cmd};
                result.stderr
                "#
                ))
                .await
                .unwrap();

            assert_eq!(result.trim(), "err");
        });
    }

    #[test]
    fn test_terminate() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                const handle = {LONG_SLEEP};
                handle.terminate();
                await handle.closed;
                "#
                ))
                .await
                .unwrap();
        });
    }

    /// Runs `process.shell()` with inherited stdio on Unix.
    /// Marked `#[ignore]` because the command writes to the real terminal.
    #[test]
    #[ignore]
    #[cfg(unix)]
    fn test_shell_unix() {
        Runtime::test_with_script_engine(async |script_engine| {
            // Verify exit code is forwarded correctly.
            let exit_code = script_engine
                .eval_async::<i32>(r#"await process.shell("exit 42")"#)
                .await
                .unwrap();
            assert_eq!(exit_code, 42);

            // Verify a successful command returns 0.
            let exit_code = script_engine
                .eval_async::<i32>(r#"await process.shell("echo hello from shell")"#)
                .await
                .unwrap();
            assert_eq!(exit_code, 0);
        });
    }

    /// Runs `process.shell()` with inherited stdio on Windows.
    /// Marked `#[ignore]` because the command writes to the real terminal.
    #[test]
    #[ignore]
    #[cfg(windows)]
    fn test_shell_windows() {
        Runtime::test_with_script_engine(async |script_engine| {
            // Verify exit code is forwarded correctly.
            let exit_code = script_engine
                .eval_async::<i32>(r#"await process.shell("exit 42")"#)
                .await
                .unwrap();
            assert_eq!(exit_code, 42);

            // Verify a successful command returns 0.
            let exit_code = script_engine
                .eval_async::<i32>(r#"await process.shell("echo hello from shell")"#)
                .await
                .unwrap();
            assert_eq!(exit_code, 0);
        });
    }

    #[test]
    #[cfg(windows)]
    fn test_terminate_gui_charmap() {
        Runtime::test_with_script_engine(async |script_engine| {
            timeout(
                Duration::from_secs(10),
                script_engine.eval_async::<()>(
                    r#"
                const handle = process.start("charmap");
                await sleep("2s");
                handle.terminate();
                await handle.closed;
                "#,
                ),
            )
            .await
            .expect("timed out waiting for charmap terminate")
            .unwrap();
        });
    }
}
