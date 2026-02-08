# Interface: DrawTextOptions

Defined in: [index.d.ts:3928](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3928)

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

> `optional` **fontSize**: `number`

Defined in: [index.d.ts:3933](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3933)

Font size in pixels.

#### Default Value

`16`

***

### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

Defined in: [index.d.ts:3943](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3943)

Horizontal alignment relative to the provided position.

#### Default Value

`TextHorizontalAlign.Left`

***

### lineSpacing?

> `optional` **lineSpacing**: `number`

Defined in: [index.d.ts:3938](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3938)

Multiplier applied to the default line height when rendering multi-line text.

#### Default Value

`1`

***

### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

Defined in: [index.d.ts:3948](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3948)

Vertical alignment relative to the provided position.

#### Default Value

`TextVerticalAlign.Top`
