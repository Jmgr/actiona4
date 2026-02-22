# Interface: ClipboardImage

Provides image clipboard operations.

```ts
const img = display.screenshot();
await clipboard.image.set(img);
const clipped = await clipboard.image.get();
```

## Methods

### get()

> <span class="async-badge">async</span> **get**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Gets the clipboard image content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

<div class="options-fields">

###### Clipboard

> **Clipboard**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

###### Selection

> **Selection**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### Platform

only works on Linux

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### set()

> <span class="async-badge">async</span> **set**(`image`: [`Image`](../classes/Image.md), `mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the clipboard image content.

#### Parameters

##### image

[`Image`](../classes/Image.md)

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

<div class="options-fields">

###### Clipboard

> **Clipboard**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

###### Selection

> **Selection**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

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
