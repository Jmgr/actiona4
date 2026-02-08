# Interface: ClickOptions

Defined in: [index.d.ts:4836](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4836)

Options for clicking a mouse button.

```ts
// Click and hold for 0.5 seconds
await mouse.click({ duration: 0.5 });
```

## Extends

- [`PressOptions`](PressOptions.md)

## Extended by

- [`DoubleClickOptions`](DoubleClickOptions.md)

## Properties

### amount?

> `optional` **amount**: `number`

Defined in: [index.d.ts:4841](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4841)

Number of times to click.

#### Default Value

`1`

***

### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

Defined in: [index.d.ts:7130](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7130)

Mouse button to press.

#### Default Value

`Button.Left`

#### Inherited from

[`PressOptions`](PressOptions.md).[`button`](PressOptions.md#button)

***

### duration?

> `optional` **duration**: `number`

Defined in: [index.d.ts:4851](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4851)

How long to hold each click, in seconds.

#### Default Value

`0`

***

### interval?

> `optional` **interval**: `number`

Defined in: [index.d.ts:4846](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4846)

Delay between consecutive clicks, in seconds.

#### Default Value

`0`

***

### position?

> `optional` **position**: [`Point`](../classes/Point.md)

Defined in: [index.d.ts:7135](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7135)

Position to move the cursor to before pressing.

#### Default Value

`undefined`

#### Inherited from

[`PressOptions`](PressOptions.md).[`position`](PressOptions.md#position)

***

### relativePosition?

> `optional` **relativePosition**: `boolean`

Defined in: [index.d.ts:7140](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7140)

Whether the position is relative to the current cursor position.

#### Default Value

`false`

#### Inherited from

[`PressOptions`](PressOptions.md).[`relativePosition`](PressOptions.md#relativeposition)

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:4856](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4856)

Abort signal to cancel the click.

#### Default Value

`undefined`
