# Interface: Process

Start and manage child processes.

```ts
const handle = process.start("echo", { args: ["hello world"] });
for await (const line of handle.stdout) {
    println(line);
}
const result = await handle.closed;
println(result.exitCode);
```

```ts
const result = await process.startAndWait("ls", { args: ["-la"] });
println(result.stdout);
```

```ts
const pid = process.startDetached("my-server", { args: ["--port", "8080"] });
println(pid);
```

## Methods

### kill()

> **kill**(`pid`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Kill a process by PID (SIGKILL on Unix, TerminateProcess on Windows).

```ts
process.kill(1234);
```

#### Parameters

##### pid

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### sendSignal()

> **sendSignal**(`pid`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `signal`: [`Signal`](../enumerations/Signal.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Send a signal to a process by PID.

```ts
process.sendSignal(1234, Signal.Term);
```

#### Parameters

##### pid

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### signal

[`Signal`](../enumerations/Signal.md)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### shell()

> <span class="async-badge">async</span> **shell**(`command`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`ShellOptions`](ShellOptions.md)): [`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Runs a command through the system shell, similar to C's `system()` function.

Stdio is inherited from the current process: if a console window is open the
command runs inside it; otherwise the OS opens a new console window for it.

The default shell is platform-specific:
- **Linux** – the value of `$SHELL`, falling back to `bash`.
- **Windows** – `powershell`.

A custom shell can be supplied via `options.shell`. On Windows the command
flag (`/C`, `-Command`, or `-c`) is inferred automatically from the shell name.

```ts
// Clear the screen (works on Windows with cmd/powershell and on Unix)
await process.shell("cls");
```

```ts
// Use a specific shell
await process.shell("echo hello", { shell: "zsh" });
```

#### Parameters

##### command

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`ShellOptions`](ShellOptions.md)

<div class="options-fields">

###### shell?

> `optional` **shell**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Shell to use. On Linux defaults to `$SHELL` (or `bash` if unset).
On Windows defaults to `powershell`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the operation.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### start()

> **start**(`command`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`StartProcessOptions`](StartProcessOptions.md)): [`ProcessHandle`](ProcessHandle.md)

Starts a process and returns a `ProcessHandle` for interacting with it.

```ts
const handle = process.start("echo", { args: ["hello world"] });
for await (const line of handle.stdout) {
    println(line);
}
const result = await handle.closed;
println(result.exitCode);
```

```ts
const handle = process.start("cat");
await handle.write("hello\n");
await handle.closeStdin();
for await (const line of handle.stdout) {
    println(line);
}
await handle.closed;
```

#### Parameters

##### command

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`StartProcessOptions`](StartProcessOptions.md)

<div class="options-fields">

###### args?

> `optional` **args**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Arguments to pass to the command.

###### Default Value

`[]`

***

###### env?

> `optional` **env**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Environment variables for the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to kill the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### workingDirectory?

> `optional` **workingDirectory**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Working directory for the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`ProcessHandle`](ProcessHandle.md)

***

### startAndWait()

> <span class="async-badge">async</span> **startAndWait**(`command`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`StartProcessOptions`](StartProcessOptions.md)): [`Task`](../type-aliases/Task.md)\<[`ProcessExitResult`](ProcessExitResult.md)\>

Starts a process, waits for it to finish, and returns the exit result
including captured stdout and stderr.

```ts
const result = await process.startAndWait("ls", { args: ["-la"] });
println(result.stdout);
println(result.exitCode);
```

#### Parameters

##### command

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`StartProcessOptions`](StartProcessOptions.md)

<div class="options-fields">

###### args?

> `optional` **args**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Arguments to pass to the command.

###### Default Value

`[]`

***

###### env?

> `optional` **env**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Environment variables for the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to kill the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### workingDirectory?

> `optional` **workingDirectory**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Working directory for the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`ProcessExitResult`](ProcessExitResult.md)\>

***

### startDetached()

> **startDetached**(`command`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`StartProcessOptions`](StartProcessOptions.md)): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Starts a detached process and returns its PID.
The process will continue running after the script exits.

```ts
const pid = process.startDetached("my-server", { args: ["--port", "8080"] });
println(`Started server with PID: ${pid}`);
```

#### Parameters

##### command

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`StartProcessOptions`](StartProcessOptions.md)

<div class="options-fields">

###### args?

> `optional` **args**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Arguments to pass to the command.

###### Default Value

`[]`

***

###### env?

> `optional` **env**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Environment variables for the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to kill the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### workingDirectory?

> `optional` **workingDirectory**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Working directory for the process.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### terminate()

> **terminate**(`pid`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Gracefully terminate a process by PID (SIGTERM on Unix, WM_CLOSE on Windows).

```ts
process.terminate(1234);
```

#### Parameters

##### pid

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
