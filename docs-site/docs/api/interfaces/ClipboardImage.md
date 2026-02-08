# Interface: ClipboardImage

Defined in: [index.d.ts:2413](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2413)

Provides image clipboard operations.

```ts
const img = display.screenshot();
await clipboard.image.set(img);
const clipped = await clipboard.image.get();
```

## Methods

### get()

> **get**(`mode?`): `Promise`\<[`Image`](../classes/Image.md)\>

Defined in: [index.d.ts:2421](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2421)

Gets the clipboard image content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<[`Image`](../classes/Image.md)\>

***

### set()

> **set**(`image`, `mode?`): `Promise`\<`void`\>

Defined in: [index.d.ts:2417](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2417)

Sets the clipboard image content.

#### Parameters

##### image

[`Image`](../classes/Image.md)

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<`void`\>
