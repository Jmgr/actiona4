# Class: Dialogs

Dialog utilities.

Provides methods for displaying message boxes and file dialogs.

```ts
const result = await dialogs.messageBox("Hello, world!");
```

```ts
const result = await dialogs.messageBox("Delete this file?", {
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

> <span class="async-badge">async</span> **messageBox**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`MessageBoxOptions`](../interfaces/MessageBoxOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

Displays a message box and returns the user's response.

```ts
const result = await dialogs.messageBox("Operation complete");
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`MessageBoxOptions`](../interfaces/MessageBoxOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the message box title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### buttons?

> `optional` **buttons?**: [`MessageBoxButtons`](MessageBoxButtons.md)

Buttons displayed in the message box.

###### Default Value

`MessageBoxButtons.ok()`

***

###### icon?

> `optional` **icon?**: [`MessageBoxIcon`](../enumerations/MessageBoxIcon.md)

<div class="options-fields">

###### Info

> **Info**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxIcon.Info`

***

###### Warning

> **Warning**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxIcon.Warning`

***

###### Error

> **Error**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxIcon.Error`

</div>

Icon displayed in the message box.

###### Default Value

`MessageBoxIcon.Info`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

***

### pickFile()

> <span class="async-badge">async</span> **pickFile**(`options?`: [`FileDialogOptions`](../interfaces/FileDialogOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a file picker dialog and returns the selected file path, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if cancelled.

```ts
const path = await dialogs.pickFile({ title: "Open File" });
if (path !== null) {
  print(path);
}
```

#### Parameters

##### options?

[`FileDialogOptions`](../interfaces/FileDialogOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### directory?

> `optional` **directory?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial directory shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](../interfaces/FileFilter.md)[]

File type filters shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### pickFiles()

> <span class="async-badge">async</span> **pickFiles**(`options?`: [`FileDialogOptions`](../interfaces/FileDialogOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

Opens a file picker dialog allowing multiple selections and returns the selected file paths.

Returns an empty array if cancelled.

```ts
const paths = await dialogs.pickFiles({ title: "Open Files" });
for (const path of paths) {
  console.log(path);
}
```

#### Parameters

##### options?

[`FileDialogOptions`](../interfaces/FileDialogOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### directory?

> `optional` **directory?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial directory shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](../interfaces/FileFilter.md)[]

File type filters shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

***

### pickFolder()

> <span class="async-badge">async</span> **pickFolder**(`options?`: [`FileDialogOptions`](../interfaces/FileDialogOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a folder picker dialog and returns the selected folder path, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if cancelled.

```ts
const path = await dialogs.pickFolder({ title: "Select Folder" });
```

#### Parameters

##### options?

[`FileDialogOptions`](../interfaces/FileDialogOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### directory?

> `optional` **directory?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial directory shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](../interfaces/FileFilter.md)[]

File type filters shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### pickFolders()

> <span class="async-badge">async</span> **pickFolders**(`options?`: [`FileDialogOptions`](../interfaces/FileDialogOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

Opens a folder picker dialog allowing multiple selections and returns the selected folder paths.

Returns an empty array if cancelled.

```ts
const paths = await dialogs.pickFolders({ title: "Select Folders" });
```

#### Parameters

##### options?

[`FileDialogOptions`](../interfaces/FileDialogOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### directory?

> `optional` **directory?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial directory shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](../interfaces/FileFilter.md)[]

File type filters shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

***

### saveFile()

> <span class="async-badge">async</span> **saveFile**(`options?`: [`FileDialogOptions`](../interfaces/FileDialogOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a save file dialog and returns the chosen file path, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if cancelled.

```ts
const path = await dialogs.saveFile({
  title: "Save As",
  filters: [{ name: "Text Files", extensions: ["txt"] }],
});
```

#### Parameters

##### options?

[`FileDialogOptions`](../interfaces/FileDialogOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### directory?

> `optional` **directory?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial directory shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](../interfaces/FileFilter.md)[]

File type filters shown in the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### textInput()

> <span class="async-badge">async</span> **textInput**(`message`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`TextInputOptions`](../interfaces/TextInputOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a text input dialog and returns the entered text, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if cancelled.

```ts
const name = await dialogs.textInput("Enter your name:", {
  title: "Name",
  mode: TextInputMode.SingleLine,
});
```

#### Parameters

##### message

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`TextInputOptions`](../interfaces/TextInputOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### value?

> `optional` **value?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial value shown in the text field.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### mode?

> `optional` **mode?**: [`TextInputMode`](../enumerations/TextInputMode.md)

<div class="options-fields">

###### SingleLine

> **SingleLine**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextInputMode.SingleLine`

***

###### MultiLine

> **MultiLine**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextInputMode.MultiLine`

***

###### Password

> **Password**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextInputMode.Password`

</div>

Input mode controlling the dialog style.

###### Default Value

`TextInputMode.SingleLine`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### colorPicker()

> <span class="async-badge">async</span> **colorPicker**(`options?`: [`ColorPickerOptions`](../interfaces/ColorPickerOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](Color.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a color picker dialog and returns the selected color, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if cancelled.

```ts
const color = await dialogs.colorPicker({
  title: "Choose a color",
  value: new Color(255, 0, 0),
});
if (color !== null) {
  print(`${color}`);
}
```

#### Parameters

##### options?

[`ColorPickerOptions`](../interfaces/ColorPickerOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### value?

> `optional` **value?**: [`ColorLike`](../type-aliases/ColorLike.md)

Initial color shown in the picker.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](Color.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `dialogs` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
