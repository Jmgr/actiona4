# Enumeration: Interpolation


Interpolation algorithms used for image rotations.

```ts
// Fast but lower quality
image.rotate(45, { interpolation: Interpolation.Nearest });

// Smooth result (default)
image.rotate(45, { interpolation: Interpolation.Bilinear });
```

## Enumeration Members

### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Nearest`

***

### Bilinear

> **Bilinear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bilinear`

***

### Bicubic

> **Bicubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bicubic`
