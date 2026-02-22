# Interface: Clipboard

The global clipboard singleton for reading and writing clipboard content.

Supports text, images, file lists, and HTML content. Each content type
is accessed through a dedicated sub-object.

```ts
// Copy and paste text
await clipboard.text.set("Hello, world!");
const text = await clipboard.text.get();

// Copy and paste an image
const img = display.screenshot();
await clipboard.image.set(img);

// Work with file lists
await clipboard.fileList.set(["/path/to/file.txt"]);

// HTML content with alt text fallback
await clipboard.html.set("<b>bold</b>", "bold");

// Clear the clipboard
await clipboard.clear();

// On Linux, use the selection clipboard
await clipboard.text.set("selected", ClipboardMode.Selection);

// Wait until clipboard content changes
await clipboard.waitForChanged();
```

## Properties

### fileList

> `readonly` **fileList**: [`ClipboardFileList`](ClipboardFileList.md)

Sub-object for file list clipboard operations.

***

### html

> `readonly` **html**: [`ClipboardHtml`](ClipboardHtml.md)

Sub-object for HTML clipboard operations.

***

### image

> `readonly` **image**: [`ClipboardImage`](ClipboardImage.md)

Sub-object for image clipboard operations.

***

### text

> `readonly` **text**: [`ClipboardText`](ClipboardText.md)

Sub-object for text clipboard operations.

## Methods

### clear()

> <span class="async-badge">async</span> **clear**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Clears the clipboard contents.

```ts
await clipboard.clear();

// On Linux, clear the selection clipboard
await clipboard.clear(ClipboardMode.Selection);
```

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

<div class="options-fields">

###### Clipboard

> **Clipboard**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ClipboardMode.Clipboard`

***

###### Selection

> **Selection**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ClipboardMode.Selection`

###### Platform

only works on Linux

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### waitForChanged()

> <span class="async-badge">async</span> **waitForChanged**(`options?`: [`WaitForChangedOptions`](WaitForChangedOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until clipboard content changes.

```ts
const controller = new AbortController();
const task = clipboard.waitForChanged({ signal: controller.signal });
// controller.abort();
await task;
```

#### Parameters

##### options?

[`WaitForChangedOptions`](WaitForChangedOptions.md)

<div class="options-fields">

###### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Polling interval in seconds.

###### Default Value

`0.2`

***

###### mode?

> `optional` **mode**: [`ClipboardMode`](../enumerations/ClipboardMode.md)

<div class="options-fields">

###### Clipboard

> **Clipboard**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ClipboardMode.Clipboard`

***

###### Selection

> **Selection**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ClipboardMode.Selection`

###### Platform

only works on Linux

</div>

Clipboard source to watch.

###### Default Value

`ClipboardMode.Clipboard`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
