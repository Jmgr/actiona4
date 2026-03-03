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

### delay?

> `optional` **delay**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delay between the two clicks, in seconds.

#### Default Value

`0.25`

***

### duration?

> `optional` **duration**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How long to hold each click, in seconds.

#### Default Value

`0`

#### Inherited from

`DoubleClickOptions`.[`duration`](#duration)

***

### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delay between consecutive clicks, in seconds.

#### Default Value

`0`

#### Inherited from

`DoubleClickOptions`.[`interval`](#interval)

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the click.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Inherited from

[`ClickOptions`](ClickOptions.md).[`signal`](ClickOptions.md#signal)
