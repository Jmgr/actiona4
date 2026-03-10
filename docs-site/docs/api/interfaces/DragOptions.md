# Interface: DragOptions


Options for drag and drop operations.

```ts
await mouse.dragAndDrop({ x: 100, y: 100 }, { x: 500, y: 500 }, {
  speed: 500,
  tween: Tween.Linear,
});
```

## Extends

- [`MoveOptions`](MoveOptions.md)

## Properties

### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

Mouse button to use for dragging.

#### Default Value

`Button.Left`

***

### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Interval in seconds

#### Default Value

`0.01`

#### Inherited from

[`MoveOptions`](MoveOptions.md).[`interval`](MoveOptions.md#interval)

***

### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

#### Default Value

`5`

#### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinAmplitude`](MoveOptions.md#perlinamplitude)

***

### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

#### Default Value

`50`

#### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinScale`](MoveOptions.md#perlinscale)

***

### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

#### Default Value

`2000`

#### Inherited from

[`MoveOptions`](MoveOptions.md).[`speed`](MoveOptions.md#speed)

***

### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

#### Default Value

`0`

#### Inherited from

[`MoveOptions`](MoveOptions.md).[`targetRandomness`](MoveOptions.md#targetrandomness)

***

### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

#### Default Value

`Tween.SineOut`

#### Inherited from

[`MoveOptions`](MoveOptions.md).[`tween`](MoveOptions.md#tween)
