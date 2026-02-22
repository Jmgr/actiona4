# Interface: ClipboardText

Provides text clipboard operations.

```ts
await clipboard.text.set("Hello!");
const text = await clipboard.text.get();
```

## Methods

### get()

> <span class="async-badge">async</span> **get**(`mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

Gets the clipboard text content.

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

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

***

### set()

> <span class="async-badge">async</span> **set**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `mode?`: [`ClipboardMode`](../enumerations/ClipboardMode.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the clipboard text content.

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

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

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
