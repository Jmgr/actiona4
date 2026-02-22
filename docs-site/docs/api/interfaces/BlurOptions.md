# Interface: BlurOptions

**`Expand`**

Options for blurring an image.

```ts
// Fast blur
image.blur({ fast: true });

// Gaussian blur with custom sigma
image.blur({ sigma: 5.0 });
```

## Properties

### fast?

> `optional` **fast**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Perform a fast, lower quality blur

#### Default Value

`false`

***

### sigma?

> `optional` **sigma**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Standard deviation of the (approximated) Gaussian

#### Default Value

`2`
