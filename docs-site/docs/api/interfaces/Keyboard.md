# Interface: Keyboard

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

### getPressedKeys()

> **getPressedKeys**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Key`](../enumerations/Key.md)[]\>

Returns the list of keys that are currently pressed.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Key`](../enumerations/Key.md)[]\>

***

### isKeyPressed()

> **isKeyPressed**(`key`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns whether a key is currently pressed.

#### Parameters

##### key

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) | [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) | [`Key`](../enumerations/Key.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

***

### key()

> **key**(`key`, `direction`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses, releases, or clicks a key.

Accepts a `Key` constant, a single character string, or a raw keycode number.

#### Parameters

##### key

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) | [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) | [`Key`](../enumerations/Key.md)

##### direction

[`Direction`](../enumerations/Direction.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### raw()

> **raw**(`keycode`, `direction`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sends a raw keycode event. Use this for keys not covered by the `Key` enum.

#### Parameters

##### keycode

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### direction

[`Direction`](../enumerations/Direction.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### text()

> **text**(`text`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Types the given text string using simulated key events.

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### waitForKeys()

> **waitForKeys**(`keys`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

([`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`Key`](../enumerations/Key.md))[]

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
