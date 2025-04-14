# Type Alias: PointLike

> **PointLike** = [`Point`](../classes/Point.md) \| \{ `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); \} \| [`Match`](../interfaces/Match.md)

A point as a [Point](../classes/Point.md) instance, a plain `{x, y}` object, or a [Match](../interfaces/Match.md).

```ts
mouse.move(new Point(100, 200)); // Point instance
mouse.move({ x: 100, y: 200 });  // plain object
mouse.move(match);               // Match from findImage
```
