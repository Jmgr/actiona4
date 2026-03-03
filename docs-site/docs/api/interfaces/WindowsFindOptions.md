# Interface: WindowsFindOptions


Window search options.

## Properties

### className?

> `optional` **className**: [`NameLike`](../type-aliases/NameLike.md)

Match by window class name.
When undefined, class name is not filtered.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### id?

> `optional` **id**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by internal window ID.
When undefined, any window ID is accepted.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### processId?

> `optional` **processId**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by window process ID.
When undefined, any process ID is accepted.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### title?

> `optional` **title**: [`NameLike`](../type-aliases/NameLike.md)

Match by window title.
When undefined, title is not filtered.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### visible?

> `optional` **visible**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Match by window visibility.
When undefined, visibility is not filtered.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
