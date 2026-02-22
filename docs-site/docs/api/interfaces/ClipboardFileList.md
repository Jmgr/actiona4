# Interface: ClipboardFileList

Provides file list clipboard operations.

```ts
await clipboard.fileList.set(["/home/user/doc.pdf", "/home/user/img.png"]);
const files = await clipboard.fileList.get();
```

## Methods

### get()

> <span class="async-badge">async</span> **get**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

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

only works on Linux

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

***

### set()

> <span class="async-badge">async</span> **set**(`fileList`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[], `mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

only works on Linux

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
