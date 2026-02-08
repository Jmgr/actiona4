# Interface: ClipboardHtml

Defined in: [index.d.ts:2456](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2456)

Provides HTML clipboard operations.

```ts
// Set HTML with a plain-text fallback
await clipboard.html.set("<b>bold</b>", "bold");

// Set HTML without a fallback
await clipboard.html.set("<em>italic</em>");

const html = await clipboard.html.get();
```

## Methods

### get()

> **get**(`mode?`): `Promise`\<`string`\>

Defined in: [index.d.ts:2464](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2464)

Gets the clipboard HTML content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<`string`\>

***

### set()

> **set**(`html`, `altText?`, `mode?`): `Promise`\<`void`\>

Defined in: [index.d.ts:2460](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2460)

Sets the clipboard HTML content, with an optional plain-text alternative.

#### Parameters

##### html

`string`

##### altText?

`string`

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<`void`\>
