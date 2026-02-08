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

> **add**(`source`, `replacement`, `options?`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

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

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### remove()

> **remove**(`source`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Removes a previously registered hotstring.

#### Parameters

##### source

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)
