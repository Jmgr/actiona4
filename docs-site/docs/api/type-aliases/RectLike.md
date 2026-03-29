# Type Alias: RectLike

> **RectLike** = [`Rect`](../classes/Rect.md) \| \{ `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); \}

A rectangle as a [Rect](../classes/Rect.md) instance or a plain `{x, y, width, height}` object.

```ts
screen.capture(new Rect(0, 0, 800, 600));                 // Rect instance
screen.capture({ x: 0, y: 0, width: 800, height: 600 }); // plain object
```
