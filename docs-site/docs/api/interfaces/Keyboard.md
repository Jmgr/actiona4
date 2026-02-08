# Interface: Keyboard

Defined in: [index.d.ts:4658](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4658)

Controls keyboard input: typing text, pressing keys, and waiting for key combinations.

```ts
// Type text
await keyboard.text("Hello, world!");
```

```ts
// Press a key combination (Ctrl+C)
await keyboard.key(Key.Control, Direction.Press);
await keyboard.key("c", Direction.Click);
await keyboard.key(Key.Control, Direction.Release);
```

```ts
// Wait for a key combination
await keyboard.waitForKeys([Key.Control, Key.Alt, "q"]);
```

## Methods

### isKeyPressed()

> **isKeyPressed**(`key`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:4676](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4676)

Returns whether a key is currently pressed.

#### Parameters

##### key

`string` | `number` | [`Key`](../enumerations/Key.md)

#### Returns

`Promise`\<`boolean`\>

***

### key()

> **key**(`key`, `direction`): `Promise`\<`void`\>

Defined in: [index.d.ts:4668](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4668)

Presses, releases, or clicks a key.

Accepts a `Key` constant, a single character string, or a raw keycode number.

#### Parameters

##### key

`string` | `number` | [`Key`](../enumerations/Key.md)

##### direction

[`Direction`](../enumerations/Direction.md)

#### Returns

`Promise`\<`void`\>

***

### raw()

> **raw**(`keycode`, `direction`): `Promise`\<`void`\>

Defined in: [index.d.ts:4672](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4672)

Sends a raw keycode event. Use this for keys not covered by the `Key` enum.

#### Parameters

##### keycode

`number`

##### direction

[`Direction`](../enumerations/Direction.md)

#### Returns

`Promise`\<`void`\>

***

### text()

> **text**(`text`): `Promise`\<`void`\>

Defined in: [index.d.ts:4662](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4662)

Types the given text string using simulated key events.

#### Parameters

##### text

`string`

#### Returns

`Promise`\<`void`\>

***

### waitForKeys()

> **waitForKeys**(`keys`): [`Task`](../type-aliases/Task.md)\<`void`\>

Defined in: [index.d.ts:4693](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4693)

Waits until the specified keys are all pressed simultaneously.

```ts
await keyboard.waitForKeys([Key.Control, "s"]);
```

```ts
// Wait for exactly these keys and no others, with abort support
const controller = new AbortController();
await keyboard.waitForKeys([Key.Control, Key.Alt, Key.Delete], {
exclusive: true,
signal: controller.signal
});
```

#### Parameters

##### keys

(`string` \| `number` \| [`Key`](../enumerations/Key.md))[]

#### Returns

[`Task`](../type-aliases/Task.md)\<`void`\>
