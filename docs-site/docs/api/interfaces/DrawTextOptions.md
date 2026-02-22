# Interface: DrawTextOptions


Options for drawing text on an image.

```ts
// Draw large, centered text
image.drawText(100, 50, "Hello", fontPath, Color.White, {
  fontSize: 32,
  horizontalAlign: TextHorizontalAlign.Center,
  verticalAlign: TextVerticalAlign.Middle
});
```

## Properties

### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

#### Default Value

`16`

***

### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

Horizontal alignment relative to the provided position.

#### Default Value

`TextHorizontalAlign.Left`

***

### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

#### Default Value

`1`

***

### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

Vertical alignment relative to the provided position.

#### Default Value

`TextVerticalAlign.Top`
