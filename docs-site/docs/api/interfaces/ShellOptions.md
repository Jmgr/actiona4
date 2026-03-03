# Interface: ShellOptions


Options for running a shell command.

## Properties

### shell?

> `optional` **shell**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Shell to use. On Linux defaults to `$SHELL` (or `bash` if unset).
On Windows defaults to `powershell`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the operation.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
