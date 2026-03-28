# Interface: DrawImageOptions


Options for drawing an image onto another image.

```ts
// Draw only a portion of the source image
canvas.drawImage(0, 0, sprite, {
  sourceRect: new Rect(0, 0, 32, 32)
});
```

## Properties

### sourceRect?

> `optional` **sourceRect?**: [`RectLike`](../type-aliases/RectLike.md)

Source rectangle.
[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) means the whole image.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
