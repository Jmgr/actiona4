# Interface: Dialogs

Dialog utilities.

Every dialog returns a task: cancelling it, aborting its `signal` or stopping the script closes
the dialog. Dialogs also accept a `timeout`, after which they close as if the user had
cancelled them; a message box then returns `MessageBoxResult.Timeout`.

```ts
const result = await dialogs.messageBox("Hello, world!");
```

```ts
const result = await dialogs.messageBox("Delete this file?", {
  title: "Confirm",
  buttons: MessageBoxButtons.YesNo,
  icon: MessageBoxIcon.Warning,
});
if (result === MessageBoxResult.Yes) {
  println("Confirmed");
}
```

```ts
// Give up waiting for an answer after 10 seconds
const name = await dialogs.textInput("Enter your name:", { timeout: "10s" });
```

## Methods

### messageBox()

> <span class="async-badge">async</span> **messageBox**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`MessageBoxOptions`](MessageBoxOptions.md)): [`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

Displays a message box and returns the button the user pressed.

Closing the message box without pressing a button returns `MessageBoxResult.Cancel` if
that button is shown, otherwise `MessageBoxResult.No`, otherwise `MessageBoxResult.Ok`.

```ts
const result = await dialogs.messageBox("Operation complete");
```

```ts
const result = await dialogs.messageBox("Save changes?", {
  buttons: MessageBoxButtons.YesNoCancel,
  labels: { yes: "Save", no: "Discard" },
  timeout: "30s",
});
if (result === MessageBoxResult.Timeout) {
  println("Nobody answered");
}
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`MessageBoxOptions`](MessageBoxOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the message box title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### buttons?

> `optional` **buttons?**: [`MessageBoxButtons`](../enumerations/MessageBoxButtons.md)

<div class="options-fields">

###### Ok

> **Ok**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxButtons.Ok`

***

###### OkCancel

> **OkCancel**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxButtons.OkCancel`

***

###### YesNo

> **YesNo**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxButtons.YesNo`

***

###### YesNoCancel

> **YesNoCancel**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`MessageBoxButtons.YesNoCancel`

</div>

Buttons displayed in the message box.

###### Default Value

`MessageBoxButtons.Ok`

***

###### labels?

> `optional` **labels?**: [`MessageBoxLabels`](MessageBoxLabels.md)

<div class="options-fields">

###### ok?

> `optional` **ok?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the OK button.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### cancel?

> `optional` **cancel?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the Cancel button.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### yes?

> `optional` **yes?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the Yes button.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### no?

> `optional` **no?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label of the No button.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

Labels replacing the default ones of the buttons.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

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

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the message box after this duration, which then returns `MessageBoxResult.Timeout`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the message box.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`MessageBoxResult`](../enumerations/MessageBoxResult.md)\>

***

### pickFile()

> <span class="async-badge">async</span> **pickFile**(`options?`: [`FileDialogOptions`](FileDialogOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a file picker dialog and returns the selected file path, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if
cancelled.

```ts
const path = await dialogs.pickFile({ title: "Open File" });
if (path !== undefined) {
  println(path);
}
```

#### Parameters

##### options?

[`FileDialogOptions`](FileDialogOptions.md)

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

###### fileName?

> `optional` **fileName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial file name. Only used by `saveFile`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](FileFilter.md)[]

File type filters shown in the dialog. Ignored when picking folders.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### pickFiles()

> <span class="async-badge">async</span> **pickFiles**(`options?`: [`FileDialogOptions`](FileDialogOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

Opens a file picker dialog allowing multiple selections and returns the selected file
paths.

Returns an empty array if cancelled.

```ts
const paths = await dialogs.pickFiles({ title: "Open Files" });
for (const path of paths) {
  println(path);
}
```

#### Parameters

##### options?

[`FileDialogOptions`](FileDialogOptions.md)

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

###### fileName?

> `optional` **fileName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial file name. Only used by `saveFile`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](FileFilter.md)[]

File type filters shown in the dialog. Ignored when picking folders.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

***

### pickFolder()

> <span class="async-badge">async</span> **pickFolder**(`options?`: [`FileDialogOptions`](FileDialogOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a folder picker dialog and returns the selected folder path, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if
cancelled.

```ts
const path = await dialogs.pickFolder({ title: "Select Folder" });
```

#### Parameters

##### options?

[`FileDialogOptions`](FileDialogOptions.md)

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

###### fileName?

> `optional` **fileName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial file name. Only used by `saveFile`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](FileFilter.md)[]

File type filters shown in the dialog. Ignored when picking folders.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### pickFolders()

> <span class="async-badge">async</span> **pickFolders**(`options?`: [`FileDialogOptions`](FileDialogOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

Opens a folder picker dialog allowing multiple selections and returns the selected
folder paths.

Returns an empty array if cancelled. Not supported by kdialog unless the
xdg-desktop-portal file chooser is available.

```ts
const paths = await dialogs.pickFolders({ title: "Select Folders" });
```

#### Parameters

##### options?

[`FileDialogOptions`](FileDialogOptions.md)

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

###### fileName?

> `optional` **fileName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial file name. Only used by `saveFile`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](FileFilter.md)[]

File type filters shown in the dialog. Ignored when picking folders.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

***

### saveFile()

> <span class="async-badge">async</span> **saveFile**(`options?`: [`FileDialogOptions`](FileDialogOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a save file dialog and returns the chosen file path, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if cancelled.

```ts
const path = await dialogs.saveFile({
  title: "Save As",
  fileName: "report.txt",
  filters: [{ name: "Text Files", extensions: ["txt"] }],
});
```

#### Parameters

##### options?

[`FileDialogOptions`](FileDialogOptions.md)

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

###### fileName?

> `optional` **fileName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial file name. Only used by `saveFile`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### filters?

> `optional` **filters?**: [`FileFilter`](FileFilter.md)[]

File type filters shown in the dialog. Ignored when picking folders.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### textInput()

> <span class="async-badge">async</span> **textInput**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`TextInputOptions`](TextInputOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a text input dialog and returns the entered text, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if cancelled.

```ts
const name = await dialogs.textInput("Enter your name:", {
  title: "Name",
  mode: TextInputMode.SingleLine,
});
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`TextInputOptions`](TextInputOptions.md)

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

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### colorPicker()

> <span class="async-badge">async</span> **colorPicker**(`options?`: [`ColorPickerOptions`](ColorPickerOptions.md)): [`Task`](../type-aliases/Task.md)\<[`Color`](../classes/Color.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Opens a color picker dialog and returns the selected color, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if cancelled.

```ts
const color = await dialogs.colorPicker({
  title: "Choose a color",
  value: new Color(255, 0, 0),
});
if (color !== undefined) {
  println(`${color}`);
}
```

#### Parameters

##### options?

[`ColorPickerOptions`](ColorPickerOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### value?

> `optional` **value?**: [`ColorLike`](../type-aliases/ColorLike.md)

Initial color shown in the picker. Its alpha channel is ignored.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Color`](../classes/Color.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### selectOne()

> <span class="async-badge">async</span> **selectOne**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `items`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[], `options?`: [`SelectOneOptions`](SelectOneOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Asks the user to pick one of `items`, and returns it, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if cancelled.

```ts
const fruit = await dialogs.selectOne("Pick a fruit:", ["Apple", "Pear", "Plum"]);
if (fruit !== undefined) {
  println(`You picked ${fruit}`);
}
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### items

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

##### options?

[`SelectOneOptions`](SelectOneOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### selected?

> `optional` **selected?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initially selected item. The first item if omitted or not one of the items.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### selectMany()

> <span class="async-badge">async</span> **selectMany**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `items`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[], `options?`: [`SelectManyOptions`](SelectManyOptions.md)): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[] \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Asks the user to pick any number of `items`, and returns them, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if
cancelled. Accepting without picking any item returns an empty array.

```ts
const fruits = await dialogs.selectMany("Pick fruits:", ["Apple", "Pear", "Plum"], {
  selected: ["Apple"],
});
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### items

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

##### options?

[`SelectManyOptions`](SelectManyOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### selected?

> `optional` **selected?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Initially selected items. Strings that are not one of the items are ignored.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[] \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### date()

> <span class="async-badge">async</span> **date**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`DateOptions`](DateOptions.md)): [`Task`](../type-aliases/Task.md)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Asks the user to pick a date, and returns it at local midnight, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if
cancelled.

```ts
const date = await dialogs.date("Pick a date:");
if (date !== undefined) {
  println(date.toDateString());
}
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`DateOptions`](DateOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### value?

> `optional` **value?**: [`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

Initially selected day; its time of day is ignored. Today if omitted.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Closes the dialog after this duration, as if the user had cancelled it.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### progress()

> <span class="async-badge">async</span> **progress**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`ProgressOptions`](ProgressOptions.md)): [`Task`](../type-aliases/Task.md)\<[`Progress`](Progress.md)\>

Opens a progress dialog, and returns it once it is shown.

The dialog stays open until `close()` is called, the `signal` is aborted, or the script
ends.

```ts
const progress = await dialogs.progress("Copying files…", { cancellable: true, value: 0 });
for (const [i, file] of files.entries()) {
  if (progress.cancelled) {
    break;
  }
  progress.text = `Copying ${file}`;
  progress.value = i / files.length;
  await copy(file);
}
await progress.close();
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`ProgressOptions`](ProgressOptions.md)

<div class="options-fields">

###### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### cancellable?

> `optional` **cancellable?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the dialog has a Cancel button.

###### Default Value

`false`

***

###### value?

> `optional` **value?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Initial progress, between 0 and 1. A busy bar is shown if omitted.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to close the dialog, both while it opens and once it is open.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Progress`](Progress.md)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `dialogs` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
