# Interface: MessageBoxOptions


Message box options.

```ts
await dialogs.messageBox("Delete this file?", {
  title: "Confirm",
  buttons: MessageBoxButtons.YesNo,
  icon: MessageBoxIcon.Warning,
});
```

## Properties

### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the message box title bar.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### buttons?

> `optional` **buttons?**: [`MessageBoxButtons`](../enumerations/MessageBoxButtons.md)

Buttons displayed in the message box.

#### Default Value

`MessageBoxButtons.Ok`

***

### labels?

> `optional` **labels?**: [`MessageBoxLabels`](MessageBoxLabels.md)

Labels replacing the default ones of the buttons.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### icon?

> `optional` **icon?**: [`MessageBoxIcon`](../enumerations/MessageBoxIcon.md)

Icon displayed in the message box.

#### Default Value

`MessageBoxIcon.Info`

***

### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the message box after this duration, which then returns `MessageBoxResult.Timeout`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the message box.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
