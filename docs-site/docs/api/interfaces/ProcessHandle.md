# Interface: ProcessHandle

A handle to a running process.

Provides access to the process's PID, stdin, stdout, stderr, and allows
waiting for the process to exit or killing it.

```ts
const handle = process.start("echo", { args: ["hello"] });
for await (const line of handle.stdout) {
    println(line);
}
const result = await handle.finished;
println(result.exitCode);
```

## Properties

### finished

> `readonly` **finished**: [`Task`](../type-aliases/Task.md)\<[`ProcessExitResult`](ProcessExitResult.md)\>

A promise that resolves with the exit result when the process finishes.

```ts
const handle = process.start("ls");
const result = await handle.finished;
println(result.exitCode);
```

***

### pid

> `readonly` **pid**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Process ID.

***

### stderr

> `readonly` **stderr**: `AsyncIterableIterator`\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

An async iterator that yields lines from the process's standard error.

```ts
const handle = process.start("my-command");
for await (const line of handle.stderr) {
    println(`error: ${line}`);
}
```

***

### stdout

> `readonly` **stdout**: `AsyncIterableIterator`\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

An async iterator that yields lines from the process's standard output.

```ts
const handle = process.start("echo", { args: ["hello"] });
for await (const line of handle.stdout) {
    println(line);
}
```

## Methods

### closeStdin()

> <span class="async-badge">async</span> **closeStdin**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Close the process's stdin. This signals EOF to the child process,
which is necessary for programs that read until EOF (like `cat`).

```ts
const handle = process.start("cat");
await handle.write("hello\n");
await handle.closeStdin();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### kill()

> <span class="async-badge">async</span> **kill**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Kill the process immediately (SIGKILL on Unix, TerminateProcess on Windows).

```ts
const handle = process.start("sleep", { args: ["100"] });
await handle.kill();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### terminate()

> <span class="async-badge">async</span> **terminate**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Gracefully terminate the process (SIGTERM on Unix, WM_CLOSE on Windows).

```ts
const handle = process.start("sleep", { args: ["100"] });
await handle.terminate();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### write()

> <span class="async-badge">async</span> **write**(`data`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Write data to the process's stdin.

```ts
const handle = process.start("cat");
await handle.write("hello\n");
```

#### Parameters

##### data

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
