# Interface: FindImageOptions

Defined in: [index.d.ts:3963](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3963)

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

> `optional` **downscale**: `number`

Defined in: [index.d.ts:3989](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3989)

How many times should the source image and the template be downscaled?

#### Default Value

`0`

***

### matchThreshold?

> `optional` **matchThreshold**: `number`

Defined in: [index.d.ts:3979](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3979)

Matching threshold.
Values are between 0 (worst) to 1 (best).

#### Default Value

`0.8`

***

### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: `number`

Defined in: [index.d.ts:3984](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3984)

Radius to consider proximity (in pixels).

#### Default Value

`10`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:3994](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3994)

Abort signal to cancel the search.

#### Default Value

`undefined`

***

### useColors?

> `optional` **useColors**: `boolean`

Defined in: [index.d.ts:3968](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3968)

Use color matching.

#### Default Value

`true`

***

### useTransparency?

> `optional` **useTransparency**: `boolean`

Defined in: [index.d.ts:3973](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3973)

Use template transparency.

#### Default Value

`true`
