# Interface: SelectOneOptions


Options for `dialogs.selectOne()`.

```ts
const fruit = await dialogs.selectOne("Pick a fruit:", ["Apple", "Pear"], {
  selected: "Pear",
});
```

## Properties

### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### selected?

> `optional` **selected?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initially selected item. The first item if omitted or not one of the items.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
