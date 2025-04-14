# Interface: ClipboardFileList

Provides file list clipboard operations.

```ts
clipboard.fileList.set(["/home/user/doc.pdf", "/home/user/img.png"]);
const files = clipboard.fileList.get();
```

## Methods

### set()

> **set**(`fileList`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[], `mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the clipboard file list content.

#### Parameters

##### fileList

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

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

### get()

> **get**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Gets the clipboard file list content.

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

readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this clipboard file list.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
