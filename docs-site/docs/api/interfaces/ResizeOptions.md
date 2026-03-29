# Interface: ResizeOptions


Options for resizing an image.

```ts
// Resize while preserving aspect ratio
image.resize(200, 150, { keepAspectRatio: true });

// Resize with a specific filter
image.resize(200, 150, { filter: ResizeFilter.Lanczos3, keepAspectRatio: true });
```

## Properties

### keepAspectRatio?

> `optional` **keepAspectRatio?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the aspect ratio be kept?

#### Default Value

`false`

***

### filter?

> `optional` **filter?**: [`ResizeFilter`](../enumerations/ResizeFilter.md)

What filter to use

#### Default Value

`ResizeFilter.Cubic`
