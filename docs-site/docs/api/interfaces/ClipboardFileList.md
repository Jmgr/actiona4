# Interface: ClipboardFileList

Provides file list clipboard operations.

```ts
await clipboard.fileList.set(["/home/user/doc.pdf", "/home/user/img.png"]);
const files = await clipboard.fileList.get();
```

## Methods

### get()

> **get**(`mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

Gets the clipboard file list content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

***

### set()

> **set**(`fileList`, `mode?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the clipboard file list content.

#### Parameters

##### fileList

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
