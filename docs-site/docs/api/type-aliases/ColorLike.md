# Type Alias: ColorLike

> **ColorLike** = [`Color`](../classes/Color.md) \| \{ `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); \}

A color as a [Color](../classes/Color.md) instance or a plain RGBA object.

```ts
image.fill(Color.Red);                              // Color instance
image.fill({ r: 255, g: 0, b: 0 });                // plain RGB object
image.fill({ r: 255, g: 0, b: 0, a: 128 });        // with alpha
```
