# Interface: ClipboardText

Defined in: [index.d.ts:2393](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2393)

Provides text clipboard operations.

```ts
await clipboard.text.set("Hello!");
const text = await clipboard.text.get();
```

## Methods

### get()

> **get**(`mode?`): `Promise`\<`string`\>

Defined in: [index.d.ts:2401](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2401)

Gets the clipboard text content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<`string`\>

***

### set()

> **set**(`text`, `mode?`): `Promise`\<`void`\>

Defined in: [index.d.ts:2397](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2397)

Sets the clipboard text content.

#### Parameters

##### text

`string`

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<`void`\>
