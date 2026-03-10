# Interface: Keyboard

Controls keyboard input: typing text, pressing keys, waiting for key combinations,
and registering text or key event listeners.

```ts
// Type text
keyboard.writeText("Hello, world!");
```

```ts
// Press a key combination (Ctrl+C)
keyboard.press(Key.Control);
keyboard.tap("c");
keyboard.release(Key.Control);
```

```ts
// Wait for a key combination
await keyboard.waitForKeys([Key.Control, Key.Alt, "q"]);
```

```ts
// Replace typed text
const h = keyboard.onText("btw", "by the way");
h.cancel(); // unregister
```

```ts
// Run a callback when a key combo is pressed
const h = keyboard.onKeys([Key.Control, Key.Alt, "t"], () => console.println("triggered!"));
```

## Methods

### clearEventHandles()

> **clearEventHandles**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Unregisters all event handles registered on this keyboard instance.

```ts
keyboard.onText("btw", "by the way");
keyboard.onKeys([Key.Control, "s"], () => save());
keyboard.clearEventHandles(); // removes both
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### getPressedKeys()

> **getPressedKeys**(): [`Key`](../enumerations/Key.md)[]

Returns the list of keys that are currently pressed.

#### Returns

[`Key`](../enumerations/Key.md)[]

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### isKeyPressed()

> **isKeyPressed**(`key`: [`KeyLike`](../type-aliases/KeyLike.md)): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns whether a key is currently pressed.

#### Parameters

##### key

[`KeyLike`](../type-aliases/KeyLike.md)

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### onKey()

> **onKey**(`key`: [`KeyLike`](../type-aliases/KeyLike.md), `callback`: [`TriggerAction`](../type-aliases/TriggerAction.md), `options?`: [`KeysOptions`](KeysOptions.md)): [`EventHandle`](EventHandle.md)

Registers a listener that fires when a single key is pressed.

```ts
const h = keyboard.onKey(Key.F5, () => console.println("F5 pressed!"));
h.cancel();
```

#### Parameters

##### key

[`KeyLike`](../type-aliases/KeyLike.md)

##### callback

[`TriggerAction`](../type-aliases/TriggerAction.md)

##### options?

[`KeysOptions`](KeysOptions.md)

<div class="options-fields">

###### exclusive?

> `optional` **exclusive**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Require exactly these keys and no others to be pressed simultaneously.

###### Default Value

`false`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the operation.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`EventHandle`](EventHandle.md)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### onKeys()

> **onKeys**(`keys`: [`Keys`](../type-aliases/Keys.md), `callback`: [`TriggerAction`](../type-aliases/TriggerAction.md), `options?`: [`KeysOptions`](KeysOptions.md)): [`EventHandle`](EventHandle.md)

Registers a listener that fires when all specified keys are pressed simultaneously.

```ts
const h = keyboard.onKeys([Key.Control, Key.Alt, "t"], () => {
  console.println("Ctrl+Alt+T pressed!");
});

// Require exactly these keys and no others
const h2 = keyboard.onKeys([Key.Control, "s"], () => save(), { exclusive: true });

h.cancel();
```

#### Parameters

##### keys

[`Keys`](../type-aliases/Keys.md)

##### callback

[`TriggerAction`](../type-aliases/TriggerAction.md)

##### options?

[`KeysOptions`](KeysOptions.md)

<div class="options-fields">

###### exclusive?

> `optional` **exclusive**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Require exactly these keys and no others to be pressed simultaneously.

###### Default Value

`false`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the operation.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`EventHandle`](EventHandle.md)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### onText()

> **onText**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `handler`: [`ReplacementHandler`](../type-aliases/ReplacementHandler.md), `options?`: [`OnTextOptions`](OnTextOptions.md)): [`EventHandle`](EventHandle.md)

Registers a listener that fires when the specified text is typed.

By default the typed text is erased and replaced with `handler`. Pass
`{ erase: false }` to trigger an action without replacing the text.

`handler` can be a string, an `Image`, a `Macro`, or a callback returning any of those.
A callback that returns nothing (void) fires without inserting anything.

```ts
// Simple text replacement
const h = keyboard.onText("btw", "by the way");

// Dynamic replacement via callback
const h = keyboard.onText("time", () => new Date().toLocaleTimeString());

// Play a macro when the trigger text is typed
const h2 = keyboard.onText("sig", loadedMacro);

// Trigger only — don't erase the typed text
const h3 = keyboard.onText("hello", () => console.println("hello typed!"), { erase: false });

h.cancel(); // unregister
```

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### handler

[`ReplacementHandler`](../type-aliases/ReplacementHandler.md)

##### options?

[`OnTextOptions`](OnTextOptions.md)

<div class="options-fields">

###### erase?

> `optional` **erase**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Erase the typed text before inserting the replacement.
Set to `false` to trigger an action without replacing the typed text.

###### Default Value

`true`

***

###### saveRestoreClipboard?

> `optional` **saveRestoreClipboard**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Save and restore the clipboard contents around a clipboard-based replacement.

###### Default Value

`true`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to automatically cancel this listener when signalled.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useClipboardForText?

> `optional` **useClipboardForText**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

When replacing with text, use the clipboard (Ctrl+V) instead of simulated keystrokes.
Replacing with an image always uses the clipboard.

###### Default Value

`false`

</div>

#### Returns

[`EventHandle`](EventHandle.md)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### press()

> **press**(`key`: [`KeyLike`](../type-aliases/KeyLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Presses and holds a key until `release` is called.

Accepts a `Key` constant, a single character string, or a raw keycode number.

#### Parameters

##### key

[`KeyLike`](../type-aliases/KeyLike.md)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### pressRaw()

> **pressRaw**(`keycode`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Presses and holds a raw keycode until `releaseRaw` is called.

Use this for keys not covered by the `Key` enum.

#### Parameters

##### keycode

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### release()

> **release**(`key`: [`KeyLike`](../type-aliases/KeyLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Releases a key previously held with `press`.

Accepts a `Key` constant, a single character string, or a raw keycode number.

#### Parameters

##### key

[`KeyLike`](../type-aliases/KeyLike.md)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### releaseRaw()

> **releaseRaw**(`keycode`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Releases a raw keycode previously held with `pressRaw`.

#### Parameters

##### keycode

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### tap()

> **tap**(`key`: [`KeyLike`](../type-aliases/KeyLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Presses and releases a key in one action.

Accepts a `Key` constant, a single character string, or a raw keycode number.

#### Parameters

##### key

[`KeyLike`](../type-aliases/KeyLike.md)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### tapRaw()

> **tapRaw**(`keycode`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Presses and releases a raw keycode in one action.

Use this for keys not covered by the `Key` enum.

#### Parameters

##### keycode

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `keyboard` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### waitForKeys()

> <span class="async-badge">async</span> **waitForKeys**(`keys`: [`Keys`](../type-aliases/Keys.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

[`Keys`](../type-aliases/Keys.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### writeText()

> **writeText**(`text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Types the given text string using simulated key events.

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>
