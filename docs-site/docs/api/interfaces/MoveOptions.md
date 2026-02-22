# Interface: MoveOptions


Options for smooth mouse movement.

```ts
await mouse.move(500, 300, {
  speed: 1000,
  tween: Tween.SineOut,
  targetRandomness: 5
});
```

## Properties

### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Interval in seconds

#### Default Value

`0.01`

***

### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

#### Default Value

`5`

***

### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

#### Default Value

`50`

***

### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

#### Default Value

`2000`

***

### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

#### Default Value

`0`

***

### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

#### Default Value

`Tween.SineOut`
