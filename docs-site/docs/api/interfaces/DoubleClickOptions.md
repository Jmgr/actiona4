# Interface: DoubleClickOptions


Options for double-clicking a mouse button.

```ts
await mouse.doubleClick({ delay: 0.1 });
```

## Extends

- [`ClickOptions`](ClickOptions.md)

## Properties

### amount?

> `optional` **amount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Number of times to click.

#### Default Value

`1`

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`amount`](ClickOptions.md#amount)

***

### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

Mouse button to press.

#### Default Value

`Button.Left`

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`button`](ClickOptions.md#button)

***

### delay?

> `optional` **delay**: [`DurationLike`](../type-aliases/DurationLike.md)

Delay between the two clicks, in seconds.

#### Default Value

`0.25`

***

### duration?

> `optional` **duration**: [`DurationLike`](../type-aliases/DurationLike.md)

How long to hold each click, in seconds.

#### Default Value

`0`

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`duration`](ClickOptions.md#duration)

***

### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Delay between consecutive clicks, in seconds.

#### Default Value

`0`

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`interval`](ClickOptions.md#interval)

***

### position?

> `optional` **position**: [`Point`](../classes/Point.md)

Position to move the cursor to before pressing.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`position`](ClickOptions.md#position)

***

### relativePosition?

> `optional` **relativePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the position is relative to the current cursor position.

#### Default Value

`false`

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`relativePosition`](ClickOptions.md#relativeposition)

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the click.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`signal`](ClickOptions.md#signal)
