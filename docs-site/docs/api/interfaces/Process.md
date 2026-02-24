# Interface: Process

Start and manage child processes.

```ts
const handle = process.start("echo", { args: ["hello world"] });
for await (const line of handle.stdout) {
    println(line);
}
const result = await handle.finished;
println(result.exitCode);
```

```ts
const result = await process.startAndWait("ls", { args: ["-la"] });
println(result.stdout);
```

```ts
const pid = await process.startDetached("my-server", { args: ["--port", "8080"] });
println(pid);
```

## Methods

### kill()

> <span class="async-badge">async</span> **kill**(`pid`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Kill a process by PID (SIGKILL on Unix, TerminateProcess on Windows).

```ts
await process.kill(1234);
```

#### Parameters

##### pid

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### sendSignal()

> <span class="async-badge">async</span> **sendSignal**(`pid`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `signal`: [`Signal`](../enumerations/Signal.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Send a signal to a process by PID.

```ts
await process.sendSignal(1234, Signal.Term);
```

#### Parameters

##### pid

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### signal

[`Signal`](../enumerations/Signal.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### start()

> **start**(`command`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`StartProcessOptions`](StartProcessOptions.md)): [`ProcessHandle`](ProcessHandle.md)

Starts a process and returns a `ProcessHandle` for interacting with it.

```ts
const handle = process.start("echo", { args: ["hello world"] });
for await (const line of handle.stdout) {
    println(line);
}
const result = await handle.finished;
println(result.exitCode);
```

```ts
const handle = process.start("cat");
await handle.write("hello\n");
await handle.closeStdin();
for await (const line of handle.stdout) {
    println(line);
}
await handle.finished;
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

> <span class="async-badge">async</span> **startDetached**(`command`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`StartProcessOptions`](StartProcessOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Starts a detached process and returns its PID.
The process will continue running after the script exits.

```ts
const pid = await process.startDetached("my-server", { args: ["--port", "8080"] });
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

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

***

### terminate()

> <span class="async-badge">async</span> **terminate**(`pid`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Gracefully terminate a process by PID (SIGTERM on Unix, WM_CLOSE on Windows).

```ts
await process.terminate(1234);
```

#### Parameters

##### pid

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
