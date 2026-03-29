# Interface: RecordOptions


Options for `macros.record()`.

```ts
const m = await macros.record({
    stopKeys: [Key.Escape],
    mousePositionInterval: "16ms",
});
```

## Properties

### stopKeys?

> `optional` **stopKeys?**: [`Key`](../enumerations/Key.md)[]

Key combination that stops the recording.
All listed keys must be pressed simultaneously.

#### Default Value

`[Key.Escape]`

***

### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Maximum recording duration before automatically stopping.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### mousePositionInterval?

> `optional` **mousePositionInterval?**: [`DurationLike`](../type-aliases/DurationLike.md)

How often to sample the mouse position.

#### Default Value

`"16ms"`

***

### mouseButtons?

> `optional` **mouseButtons?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record mouse button press and release events.

#### Default Value

`true`

***

### mousePosition?

> `optional` **mousePosition?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record mouse cursor position.

#### Default Value

`true`

***

### mouseScroll?

> `optional` **mouseScroll?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record mouse scroll wheel events.

#### Default Value

`true`

***

### keyboardKeys?

> `optional` **keyboardKeys?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Record keyboard key press and release events.

#### Default Value

`true`

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel recording.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
