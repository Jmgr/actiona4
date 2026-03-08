# Interface: Macros

Records and replays input macros.

```ts
// Record until Escape is pressed
const m = await macros.record();
await macros.play(m);
```

```ts
// Save and reload a macro
const m = await macros.record({ timeout: "30s" });
await m.save("workflow.amacro");
const loaded = await Macro.load("workflow.amacro");
await macros.play(loaded, { speed: 2.0 });
```

## Methods

### play()

> <span class="async-badge">async</span> **play**(`macroArg`: [`Macro`](../classes/Macro.md), `options?`: [`PlayOptions`](PlayOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void), [`PlayProgress`](../classes/PlayProgress.md)\>

Replays a previously recorded macro.

Only one playback can be active at a time; calling `play()` while another
playback is already running throws an error.

```ts
await macros.play(macro);
```

```ts
// Play at twice the original speed, skipping mouse movements
await macros.play(macro, { speed: 2.0, mousePosition: false });
```

```ts
// Cancellable playback with progress tracking
const controller = new AbortController();
const task = macros.play(macro, { signal: controller.signal });
for await (const progress of task) {
    console.println(`${Math.round(progress.ratio() * 100)}%`);
    if (progress.finished()) break;
}
await task;
```

#### Parameters

##### macroArg

[`Macro`](../classes/Macro.md)

##### options?

[`PlayOptions`](PlayOptions.md)

<div class="options-fields">

###### keyboardKeys?

> `optional` **keyboardKeys**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay keyboard key events.

###### Default Value

`true`

***

###### mouseButtons?

> `optional` **mouseButtons**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse button events.

###### Default Value

`true`

***

###### mousePosition?

> `optional` **mousePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse cursor movements.

###### Default Value

`true`

***

###### mouseScroll?

> `optional` **mouseScroll**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse scroll events.

###### Default Value

`true`

***

###### relativeMousePosition?

> `optional` **relativeMousePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse movements relative to the current cursor position instead of absolute
screen coordinates. The offset is computed from the difference between the cursor's
position at playback start and the first recorded mouse position.

###### Default Value

`false`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel playback.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Playback speed multiplier. `1.0` is real-time, `2.0` is twice as fast.
Must be greater than zero.

###### Default Value

`1`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void), [`PlayProgress`](../classes/PlayProgress.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### record()

> <span class="async-badge">async</span> **record**(`options?`: [`RecordOptions`](RecordOptions.md)): [`Task`](../type-aliases/Task.md)\<[`Macro`](../classes/Macro.md)\>

Records user input until the stop key combination is pressed (or the timeout elapses).

```ts
// Record with default settings (stop with Escape)
const m = await macros.record();
```

```ts
// Record with a 30-second timeout
const m = await macros.record({ timeout: "30s" });
```

```ts
// Record only keyboard events
const m = await macros.record({
    mouseButtons: false,
    mousePosition: false,
    mouseScroll: false,
});
```

#### Parameters

##### options?

[`RecordOptions`](RecordOptions.md)

<div class="options-fields">

###### keyboardKeys?

> `optional` **keyboardKeys**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record keyboard key press and release events.

###### Default Value

`true`

***

###### mouseButtons?

> `optional` **mouseButtons**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record mouse button press and release events.

###### Default Value

`true`

***

###### mousePosition?

> `optional` **mousePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record mouse cursor position.

###### Default Value

`true`

***

###### mousePositionInterval?

> `optional` **mousePositionInterval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How often to sample the mouse position.

###### Default Value

`"16ms"`

***

###### mouseScroll?

> `optional` **mouseScroll**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record mouse scroll wheel events.

###### Default Value

`true`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel recording.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### stopKeys?

> `optional` **stopKeys**: [`Key`](../enumerations/Key.md)[]

Key combination that stops the recording.
All listed keys must be pressed simultaneously.

###### Default Value

`[Key.Escape]`

***

###### timeout?

> `optional` **timeout**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Maximum recording duration before automatically stopping.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Macro`](../classes/Macro.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
