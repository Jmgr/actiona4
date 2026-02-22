# Interface: StartProcessOptions


Options for starting a process.

## Properties

### args?

> `optional` **args**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Arguments to pass to the command.

#### Default Value

`[]`

***

### env?

> `optional` **env**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Environment variables for the process.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to kill the process.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### workingDirectory?

> `optional` **workingDirectory**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Working directory for the process.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
