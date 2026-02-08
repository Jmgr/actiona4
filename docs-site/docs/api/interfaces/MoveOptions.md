# Interface: MoveOptions

Defined in: [index.d.ts:7081](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7081)

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

> `optional` **interval**: `number`

Defined in: [index.d.ts:7111](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7111)

Interval in seconds

#### Default Value

`0.01`

***

### perlinAmplitude?

> `optional` **perlinAmplitude**: `number`

Defined in: [index.d.ts:7101](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7101)

Amplitude of the Perlin noise applied to the movement path.

#### Default Value

`5`

***

### perlinScale?

> `optional` **perlinScale**: `number`

Defined in: [index.d.ts:7096](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7096)

Scale of the Perlin noise applied to the movement path.

#### Default Value

`50`

***

### speed?

> `optional` **speed**: `number`

Defined in: [index.d.ts:7086](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7086)

Movement speed in pixels per second.

#### Default Value

`2000`

***

### targetRandomness?

> `optional` **targetRandomness**: `number`

Defined in: [index.d.ts:7106](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7106)

Random offset applied to the target position, in pixels.

#### Default Value

`0`

***

### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Defined in: [index.d.ts:7091](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7091)

Easing function used for the movement.

#### Default Value

`Tween.SineOut`
