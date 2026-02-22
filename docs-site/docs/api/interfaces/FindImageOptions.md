# Interface: FindImageOptions

**`Expand`**

Options for finding an image within another image.

```ts
// Find with stricter matching
const match = await source.findImage(template, { matchThreshold: 0.95 });

// Find with abort support
const controller = new AbortController();
const match = await source.findImage(template, { signal: controller.signal });
```

## Properties

### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

#### Default Value

`0`

***

### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

#### Default Value

`0.8`

***

### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

#### Default Value

`10`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

#### Default Value

`true`

***

### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

#### Default Value

`true`
