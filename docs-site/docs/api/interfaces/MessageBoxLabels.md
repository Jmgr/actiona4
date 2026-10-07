# Interface: MessageBoxLabels


Labels replacing the default ones of the message box buttons. A label is only used if its
button is shown.

```ts
await dialogs.messageBox("Save changes?", {
  buttons: MessageBoxButtons.YesNoCancel,
  labels: { yes: "Save", no: "Discard" },
});
```

## Properties

### ok?

> `optional` **ok?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the OK button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### cancel?

> `optional` **cancel?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the Cancel button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### yes?

> `optional` **yes?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the Yes button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### no?

> `optional` **no?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the No button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
