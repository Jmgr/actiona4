# Interface: MeasureSpeedOptions

Options for measuring mouse movement speed.

```ts
const speed = await mouse.measureSpeed({ duration: 3 });
```

## Properties

### duration?

> `optional` **duration**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Duration in seconds

#### Default Value

`2`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the measurement.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
