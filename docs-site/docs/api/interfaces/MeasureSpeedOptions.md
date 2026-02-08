# Interface: MeasureSpeedOptions

Defined in: [index.d.ts:4815](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4815)

Options for measuring mouse movement speed.

```ts
const speed = await mouse.measureSpeed({ duration: 3 });
```

## Properties

### duration?

> `optional` **duration**: `number`

Defined in: [index.d.ts:4820](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4820)

Duration in seconds

#### Default Value

`2`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:4825](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4825)

Abort signal to cancel the measurement.

#### Default Value

`undefined`
