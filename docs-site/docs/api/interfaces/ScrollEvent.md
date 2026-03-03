# Interface: ScrollEvent

The result of a `waitForScroll` call.

```ts
const event = await mouse.waitForScroll();
console.println(`Scrolled ${event.length} on axis ${event.axis}`);
```

## Properties

### axis

> `readonly` **axis**: [`Axis`](../enumerations/Axis.md)

The scroll axis.

***

### length

> `readonly` **length**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The scroll amount. Positive values scroll down/right, negative values scroll up/left.

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
