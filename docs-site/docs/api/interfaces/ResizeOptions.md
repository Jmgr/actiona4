# Interface: ResizeOptions

Defined in: [index.d.ts:3812](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3812)

Options for resizing an image.

```ts
// Resize while preserving aspect ratio
image.resize(200, 150, { keepAspectRatio: true });

// Resize with a specific filter
image.resize(200, 150, { filter: ResizeFilter.Lanczos3, keepAspectRatio: true });
```

## Properties

### filter?

> `optional` **filter**: [`ResizeFilter`](../enumerations/ResizeFilter.md)

Defined in: [index.d.ts:3822](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3822)

What filter to use

#### Default Value

`ResizeFilter.Cubic`

***

### keepAspectRatio?

> `optional` **keepAspectRatio**: `boolean`

Defined in: [index.d.ts:3817](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3817)

Should the aspect ratio be kept?

#### Default Value

`false`
