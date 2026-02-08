# Interface: DrawingOptions

Options for drawing shapes on an image.

```ts
// Draw a hollow circle (outline only)
image.drawCircle(50, 50, 20, Color.Red, { hollow: true });
```

## Properties

### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

#### Default Value

`false`
