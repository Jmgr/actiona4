# Interface: ClickOptions


Options for clicking a mouse button.

```ts
// Click and hold for 0.5 seconds
await mouse.click({ duration: 0.5 });
```

## Extends

- `PressOptions`

## Extended by

- [`DoubleClickOptions`](DoubleClickOptions.md)

## Properties

### amount?

> `optional` **amount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Number of times to click.

#### Default Value

`1`

***

### duration?

> `optional` **duration**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How long to hold each click, in seconds.

#### Default Value

`0`

***

### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delay between consecutive clicks, in seconds.

#### Default Value

`0`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the click.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
