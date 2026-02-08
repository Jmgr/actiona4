# Interface: ClipboardHtml

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

> **get**(`mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

Gets the clipboard HTML content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

***

### set()

> **set**(`html`, `altText?`, `mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the clipboard HTML content, with an optional plain-text alternative.

#### Parameters

##### html

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### altText?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
