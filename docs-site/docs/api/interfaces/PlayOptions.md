# Interface: PlayOptions


Options for `macros.play()`.

```ts
await macros.play(macro, { speed: 2.0 });
```

## Properties

### speed?

> `optional` **speed?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Playback speed multiplier. `1.0` is real-time, `2.0` is twice as fast.
Must be greater than zero.

#### Default Value

`1`

***

### mouseButtons?

> `optional` **mouseButtons?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse button events.

#### Default Value

`true`

***

### mousePosition?

> `optional` **mousePosition?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse cursor movements.

#### Default Value

`true`

***

### relativeMousePosition?

> `optional` **relativeMousePosition?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse movements relative to the current cursor position instead of absolute
screen coordinates. The offset is computed from the difference between the cursor's
position at playback start and the first recorded mouse position.

#### Default Value

`false`

***

### mouseScroll?

> `optional` **mouseScroll?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay mouse scroll events.

#### Default Value

`true`

***

### keyboardKeys?

> `optional` **keyboardKeys?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Replay keyboard key events.

#### Default Value

`true`

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel playback.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
