# Class: Ui

Defined in: [index.d.ts:6563](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6563)

User interface utilities.

Provides methods for displaying message boxes and other UI elements.
Only available when running with the Tauri UI.

```ts
const result = await Ui.messageBox("Hello, world!");
```

```ts
const result = await Ui.messageBox("Delete this file?", {
title: "Confirm",
buttons: MessageBoxButtons.yesNo(),
icon: MessageBoxIcon.Warning,
});
if (result === MessageBoxResult.Yes) {
console.log("Confirmed");
}
```

## Methods

### messageBox()

> `static` **messageBox**(`text`, `options?`): [`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

Defined in: [index.d.ts:6572](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6572)

Displays a message box and returns the user's response.

```ts
const result = await Ui.messageBox("Operation complete");
```

#### Parameters

##### text

`string`

##### options?

[`MessageBoxOptions`](../interfaces/MessageBoxOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>
