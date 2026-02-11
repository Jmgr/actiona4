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

> **clear**(`mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Clears the clipboard contents.

```ts
await clipboard.clear();

// On Linux, clear the selection clipboard
await clipboard.clear(ClipboardMode.Selection);
```

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### waitForChanged()

> **waitForChanged**(`options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
