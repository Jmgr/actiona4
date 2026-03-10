# Interface: ProcessExitResult

The result of a process that has finished.

```ts
const handle = process.start("ls");
const result = await handle.closed;
if (result.exitCode === 0) {
    println("success");
}
```

```ts
const result = await process.startAndWait("echo", { args: ["hello"] });
println(result.stdout);
```

## Properties

### exitCode?

> `readonly` `optional` **exitCode**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The exit code of the process. [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if the process was killed by a signal.

***

### pid?

> `readonly` `optional` **pid**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The process ID. Only available when using `handle.closed`.

***

### stderr?

> `readonly` `optional` **stderr**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The captured stderr output. Only available when using `startAndWait`.

***

### stdout?

> `readonly` `optional` **stdout**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The captured stdout output. Only available when using `startAndWait`.

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this process exit result.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
