# Interface: DrawImageOptions

Defined in: [index.d.ts:3859](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3859)

Options for drawing an image onto another image.

```ts
// Draw only a portion of the source image
canvas.drawImage(0, 0, sprite, {
sourceRect: new Rect(0, 0, 32, 32)
});
```

## Properties

### sourceRect?

> `optional` **sourceRect**: [`Rect`](../classes/Rect.md)

Defined in: [index.d.ts:3865](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3865)

Source rectangle.
`undefined` means the whole image.

#### Default Value

`undefined`
