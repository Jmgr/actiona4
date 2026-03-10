# Interface: Clipboard

The global clipboard singleton for reading and writing clipboard content.

Supports text, images, file lists, and HTML content. Each content type
is accessed through a dedicated sub-object.

```ts
// Copy and paste text
clipboard.text.set("Hello, world!");
const text = clipboard.text.get();

// Copy and paste an image
const img = screen.captureDesktop();
clipboard.image.set(img);

// Work with file lists
clipboard.fileList.set(["/path/to/file.txt"]);

// HTML content with alt text fallback
clipboard.html.set("<b>bold</b>", "bold");

// Clear the clipboard
clipboard.clear();

// On Linux, use the selection clipboard
clipboard.text.set("selected", ClipboardMode.Selection);

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

> **clear**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Clears the clipboard contents.

```ts
clipboard.clear();

// On Linux, clear the selection clipboard
clipboard.clear(ClipboardMode.Selection);
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

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

</div>

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `clipboard` singleton.

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

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

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

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

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
