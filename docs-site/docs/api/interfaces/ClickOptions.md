# Interface: ClickOptions


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

> `optional` **amount?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Number of times to click.

#### Default Value

`1`

***

### button?

> `optional` **button?**: [`Button`](../enumerations/Button.md)

Mouse button to press.

#### Default Value

`Button.Left`

#### Inherited from

[`PressOptions`](PressOptions.md).[`button`](PressOptions.md#button)

***

### duration?

> `optional` **duration?**: [`DurationLike`](../type-aliases/DurationLike.md)

How long to hold each click, in seconds.

#### Default Value

`0`

***

### interval?

> `optional` **interval?**: [`DurationLike`](../type-aliases/DurationLike.md)

Delay between consecutive clicks, in seconds.

#### Default Value

`0`

***

### position?

> `optional` **position?**: [`PointLike`](../type-aliases/PointLike.md)

Position to move the cursor to before pressing.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Inherited from

[`PressOptions`](PressOptions.md).[`position`](PressOptions.md#position)

***

### relativePosition?

> `optional` **relativePosition?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the position is relative to the current cursor position.

#### Default Value

`false`

#### Inherited from

[`PressOptions`](PressOptions.md).[`relativePosition`](PressOptions.md#relativeposition)

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the click.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
