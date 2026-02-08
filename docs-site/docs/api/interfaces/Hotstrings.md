# Interface: Hotstrings

Defined in: [index.d.ts:3779](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3779)

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

> **add**(`source`, `replacement`, `options?`): `void`

Defined in: [index.d.ts:3790](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3790)

Registers a hotstring. When the user types `source`, it is replaced with `replacement`.

The replacement can be a string, an `Image`, or a callback returning either.

```ts
// With options: don't erase the typed key
hotstrings.add("sig", "Best regards,\nJohn", { eraseKey: false });
```

#### Parameters

##### source

`string`

##### replacement

`string` | [`Image`](../classes/Image.md) | () => `string` \| `Promise`\<`string`\> | () => [`Image`](../classes/Image.md) \| `Promise`\<[`Image`](../classes/Image.md)\>

##### options?

[`HotstringOptions`](HotstringOptions.md)

#### Returns

`void`

***

### remove()

> **remove**(`source`): `void`

Defined in: [index.d.ts:3794](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3794)

Removes a previously registered hotstring.

#### Parameters

##### source

`string`

#### Returns

`void`
