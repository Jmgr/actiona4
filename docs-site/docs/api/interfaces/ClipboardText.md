# Interface: ClipboardText

Provides text clipboard operations.

```ts
await clipboard.text.set("Hello!");
const text = await clipboard.text.get();
```

## Methods

### get()

> **get**(`mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

Gets the clipboard text content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

***

### set()

> **set**(`text`, `mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the clipboard text content.

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
