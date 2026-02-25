# Interface: ClipboardImage

Provides image clipboard operations.

```ts
const img = display.screenshot();
clipboard.image.set(img);
const clipped = clipboard.image.get();
```

## Methods

### get()

> **get**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Image`](../classes/Image.md)

Gets the clipboard image content.

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
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

</div>

#### Returns

[`Image`](../classes/Image.md)

***

### set()

> **set**(`image`: [`Image`](../classes/Image.md), `mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the clipboard image content.

#### Parameters

##### image

[`Image`](../classes/Image.md)

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
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

</div>

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
