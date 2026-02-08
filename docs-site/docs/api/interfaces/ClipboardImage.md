# Interface: ClipboardImage

Provides image clipboard operations.

```ts
const img = display.screenshot();
await clipboard.image.set(img);
const clipped = await clipboard.image.get();
```

## Methods

### get()

> **get**(`mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Gets the clipboard image content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### set()

> **set**(`image`, `mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the clipboard image content.

#### Parameters

##### image

[`Image`](../classes/Image.md)

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
