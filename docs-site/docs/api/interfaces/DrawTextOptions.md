# Interface: DrawTextOptions


Options for drawing text on an image.

```ts
// Draw large, centered text with default font
image.drawText(100, 50, "Hello", Color.White, {
  fontSize: 32,
  horizontalAlign: TextHorizontalAlign.Center,
  verticalAlign: TextVerticalAlign.Middle
});

// Draw text with a custom font
const font = await Font.load("/path/to/font.ttf");
image.drawText(100, 50, "Hello", Color.White, { font, fontSize: 32 });
```

## Properties

### font?

> `optional` **font?**: [`Font`](../classes/Font.md)

Font to use. Defaults to the built-in DejaVu Sans.

#### Default Value

`Font.default()`

***

### fontSize?

> `optional` **fontSize?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

#### Default Value

`16`

***

### lineSpacing?

> `optional` **lineSpacing?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

#### Default Value

`1`

***

### horizontalAlign?

> `optional` **horizontalAlign?**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

Horizontal alignment relative to the provided position.

#### Default Value

`TextHorizontalAlign.Left`

***

### verticalAlign?

> `optional` **verticalAlign?**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

Vertical alignment relative to the provided position.

#### Default Value

`TextVerticalAlign.Top`
