# Class: Ui

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
  println("Confirmed");
}
```

## Methods

### messageBox()

> `static` **messageBox**(`text`, `options?`): [`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

Displays a message box and returns the user's response.

```ts
const result = await Ui.messageBox("Operation complete");
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`MessageBoxOptions`](../interfaces/MessageBoxOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>
