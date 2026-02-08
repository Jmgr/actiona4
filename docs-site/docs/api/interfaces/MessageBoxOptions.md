# Interface: MessageBoxOptions

Defined in: [index.d.ts:6519](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6519)

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

Defined in: [index.d.ts:6529](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6529)

Buttons displayed in the message box.

#### Default Value

`MessageBoxButtons.ok()`

***

### icon?

> `optional` **icon**: [`MessageBoxIcon`](../enumerations/MessageBoxIcon.md)

Defined in: [index.d.ts:6534](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6534)

Icon displayed in the message box.

#### Default Value

`MessageBoxIcon.Info`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:6539](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6539)

Abort signal to cancel the message box.

#### Default Value

`undefined`

***

### title?

> `optional` **title**: `string`

Defined in: [index.d.ts:6524](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6524)

Title displayed in the message box title bar.

#### Default Value

`undefined`
