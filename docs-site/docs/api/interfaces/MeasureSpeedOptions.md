# Interface: MeasureSpeedOptions

Options for measuring mouse movement speed.

```ts
const speed = await mouse.measureSpeed({ duration: "3s" });
```

## Properties

### duration?

> `optional` **duration**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Measurement duration.

#### Default Value

`2s`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the measurement.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
