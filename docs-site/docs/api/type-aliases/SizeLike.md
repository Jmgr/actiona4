# Type Alias: SizeLike

> **SizeLike** = [`Size`](../classes/Size.md) \| \{ `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number); \}

A size as a [Size](../classes/Size.md) instance or a plain `{width, height}` object.

```ts
image.resize(new Size(800, 600));           // Size instance
image.resize({ width: 800, height: 600 }); // plain object
```
