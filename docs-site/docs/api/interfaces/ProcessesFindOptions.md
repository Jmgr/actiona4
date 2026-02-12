# Interface: ProcessesFindOptions

Process search options.

## Properties

### name?

> `optional` **name**: [`NameLike`](../type-aliases/NameLike.md)

Match by process name.
When undefined, name is not filtered.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### parentPid?

> `optional` **parentPid**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by parent process ID.
When undefined, parent PID is not filtered.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### pid?

> `optional` **pid**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by process ID.
When undefined, any PID is accepted.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### rescan?

> `optional` **rescan**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Refresh process list before filtering.

#### Default Value

`true`

***

### status?

> `optional` **status**: [`ProcessStatus`](../enumerations/ProcessStatus.md)

Match by process status.
When undefined, status is not filtered.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
