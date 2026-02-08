# Interface: MessageBoxOptions

Message box options.

```ts
await Ui.messageBox("Delete this file?", {
title: "Confirm",
buttons: MessageBoxButtons.yesNo(),
icon: MessageBoxIcon.Warning,
});
```

## Properties

### buttons?

> `optional` **buttons**: [`MessageBoxButtons`](../classes/MessageBoxButtons.md)

Buttons displayed in the message box.

#### Default Value

`MessageBoxButtons.ok()`

***

### icon?

> `optional` **icon**: [`MessageBoxIcon`](../enumerations/MessageBoxIcon.md)

Icon displayed in the message box.

#### Default Value

`MessageBoxIcon.Info`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the message box.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### title?

> `optional` **title**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the message box title bar.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
