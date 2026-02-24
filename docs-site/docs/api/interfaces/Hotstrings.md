# Interface: Hotstrings

The global hotstrings singleton for registering text-replacement triggers.

When the user types a registered source string, it is automatically replaced
with the specified replacement (text, callback, or image).

```ts
// Simple text replacement
hotstrings.add("btw", "by the way");

// Dynamic replacement via callback
hotstrings.add("time", () => new Date().toLocaleTimeString());

// Async callback
hotstrings.add("rand", async () => "" + random.integer(0, 99999));

// Remove a hotstring
hotstrings.remove("btw");
```

## Methods

### add()

> **add**(`source`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `replacement`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`Image`](../classes/Image.md) \| () => [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\> \| () => [`Image`](../classes/Image.md) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>, `options?`: [`HotstringOptions`](HotstringOptions.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Registers a hotstring. When the user types `source`, it is replaced with `replacement`.

The replacement can be a string, an `Image`, or a callback returning either.

```ts
// With options: don't erase the typed key
hotstrings.add("sig", "Best regards,\nJohn", { eraseKey: false });
```

#### Parameters

##### source

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### replacement

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) | [`Image`](../classes/Image.md) | () => [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\> | () => [`Image`](../classes/Image.md) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

##### options?

[`HotstringOptions`](HotstringOptions.md)

<div class="options-fields">

###### eraseKey?

> `optional` **eraseKey**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Erase the key first before replacing it with the replacement content.

###### Default Value

`true`

***

###### saveRestoreClipboard?

> `optional` **saveRestoreClipboard**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Try to save and restore the clipboard's contents.

###### Default Value

`true`

***

###### useClipboardForText?

> `optional` **useClipboardForText**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

When replacing with text, save it to the clipboard then simulate Ctrl+V to paste.
Replacing with an image always uses the clipboard.

###### Default Value

`false`

</div>

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### clear()

> **clear**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Removes all registered hotstrings.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### remove()

> **remove**(`source`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Removes a previously registered hotstring.

#### Parameters

##### source

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
