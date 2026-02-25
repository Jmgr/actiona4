# Interface: Keyboard

Controls keyboard input: typing text, pressing keys, and waiting for key combinations.

```ts
// Type text
keyboard.text("Hello, world!");
```

```ts
// Press a key combination (Ctrl+C)
keyboard.key(Key.Control, Direction.Press);
keyboard.key("c", Direction.Click);
keyboard.key(Key.Control, Direction.Release);
```

```ts
// Wait for a key combination
await keyboard.waitForKeys([Key.Control, Key.Alt, "q"]);
```

## Methods

### getPressedKeys()

> **getPressedKeys**(): [`Key`](../enumerations/Key.md)[]

Returns the list of keys that are currently pressed.

#### Returns

[`Key`](../enumerations/Key.md)[]

***

### isKeyPressed()

> **isKeyPressed**(`key`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`Key`](../enumerations/Key.md)): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns whether a key is currently pressed.

#### Parameters

##### key

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) | [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) | [`Key`](../enumerations/Key.md)

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### key()

> **key**(`key`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`Key`](../enumerations/Key.md), `direction`: [`Direction`](../enumerations/Direction.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Presses, releases, or clicks a key.

Accepts a `Key` constant, a single character string, or a raw keycode number.

#### Parameters

##### key

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) | [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) | [`Key`](../enumerations/Key.md)

##### direction

[`Direction`](../enumerations/Direction.md)

<div class="options-fields">

###### Click

> **Click**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Click`

***

###### Press

> **Press**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Press`

***

###### Release

> **Release**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Release`

</div>

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### raw()

> **raw**(`keycode`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `direction`: [`Direction`](../enumerations/Direction.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sends a raw keycode event. Use this for keys not covered by the `Key` enum.

#### Parameters

##### keycode

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### direction

[`Direction`](../enumerations/Direction.md)

<div class="options-fields">

###### Click

> **Click**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Click`

***

###### Press

> **Press**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Press`

***

###### Release

> **Release**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Release`

</div>

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### text()

> **text**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Types the given text string using simulated key events.

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### waitForKeys()

> <span class="async-badge">async</span> **waitForKeys**(`keys`: ([`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`Key`](../enumerations/Key.md))[]): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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
