# Interface: MeasureSpeedOptions


Options for measuring mouse movement speed.

```ts
const speed = await mouse.measureSpeed({ duration: "3s" });
```

## Properties

### duration?

> `optional` **duration**: [`DurationLike`](../type-aliases/DurationLike.md)

Measurement duration.

#### Default Value

`2s`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the measurement.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
