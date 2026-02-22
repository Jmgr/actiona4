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

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### messageBox()

> <span class="async-badge">async</span> `static` **messageBox**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`MessageBoxOptions`](../interfaces/MessageBoxOptions.md)): [`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

Displays a message box and returns the user's response.

```ts
const result = await Ui.messageBox("Operation complete");
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`MessageBoxOptions`](../interfaces/MessageBoxOptions.md)

<div class="options-fields">

###### buttons?

> `optional` **buttons**: [`MessageBoxButtons`](MessageBoxButtons.md)

Buttons displayed in the message box.

###### Default Value

`MessageBoxButtons.ok()`

***

###### icon?

> `optional` **icon**: [`MessageBoxIcon`](../enumerations/MessageBoxIcon.md)

<div class="options-fields">

###### Error

> **Error**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxIcon.Error`

***

###### Info

> **Info**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxIcon.Info`

***

###### Warning

> **Warning**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxIcon.Warning`

</div>

Icon displayed in the message box.

###### Default Value

`MessageBoxIcon.Info`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

Abort signal to cancel the message box.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### title?

> `optional` **title**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the message box title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>
