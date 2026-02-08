# Interface: Clipboard

Defined in: [index.d.ts:2351](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2351)

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
```

## Properties

### fileList

> `readonly` **fileList**: [`ClipboardFileList`](ClipboardFileList.md)

Defined in: [index.d.ts:2363](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2363)

Sub-object for file list clipboard operations.

***

### html

> `readonly` **html**: [`ClipboardHtml`](ClipboardHtml.md)

Defined in: [index.d.ts:2367](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2367)

Sub-object for HTML clipboard operations.

***

### image

> `readonly` **image**: [`ClipboardImage`](ClipboardImage.md)

Defined in: [index.d.ts:2359](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2359)

Sub-object for image clipboard operations.

***

### text

> `readonly` **text**: [`ClipboardText`](ClipboardText.md)

Defined in: [index.d.ts:2355](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2355)

Sub-object for text clipboard operations.

## Methods

### clear()

> **clear**(`mode?`): `Promise`\<`void`\>

Defined in: [index.d.ts:2378](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2378)

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

`Promise`\<`void`\>
