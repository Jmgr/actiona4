# Interface: BlurOptions

Defined in: [index.d.ts:3836](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3836)

Options for blurring an image.

```ts
// Fast blur
image.blur({ fast: true });

// Gaussian blur with custom sigma
image.blur({ sigma: 5.0 });
```

## Properties

### fast?

> `optional` **fast**: `boolean`

Defined in: [index.d.ts:3841](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3841)

Perform a fast, lower quality blur

#### Default Value

`false`

***

### sigma?

> `optional` **sigma**: `number`

Defined in: [index.d.ts:3846](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3846)

Standard deviation of the (approximated) Gaussian

#### Default Value

`2`
