# Enumeration: ResizeFilter


Resize filter algorithms.

```ts
// Use nearest-neighbor for pixel art (no smoothing)
image.resize(64, 64, { filter: ResizeFilter.Nearest });

// Use Lanczos3 for high-quality downscaling
image.resize(200, 150, { filter: ResizeFilter.Lanczos3 });
```

## Enumeration Members

### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Nearest`

***

### Linear

> **Linear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Linear`

***

### Cubic

> **Cubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Cubic`

***

### Gaussian

> **Gaussian**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Gaussian`

***

### Lanczos3

> **Lanczos3**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Lanczos3`
