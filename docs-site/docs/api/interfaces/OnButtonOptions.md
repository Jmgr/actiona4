# Interface: OnButtonOptions


Options for `onButton`.

## Properties

### exclusive?

> `optional` **exclusive**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Require exactly this button and no others to be pressed simultaneously.

#### Default Value

`false`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to automatically cancel this listener when signalled.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
