# Interface: MessageBoxOptions


Message box options.

```ts
await ui.messageBox("Delete this file?", {
  title: "Confirm",
  buttons: MessageBoxButtons.yesNo(),
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

> `optional` **buttons?**: [`MessageBoxButtons`](../classes/MessageBoxButtons.md)

Buttons displayed in the message box.

#### Default Value

`MessageBoxButtons.ok()`

***

### icon?

> `optional` **icon?**: [`MessageBoxIcon`](../enumerations/MessageBoxIcon.md)

Icon displayed in the message box.

#### Default Value

`MessageBoxIcon.Info`
