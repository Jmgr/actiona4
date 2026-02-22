# Enumeration: Interpolation


Interpolation algorithms used for image rotations.

```ts
// Fast but lower quality
image.rotate(45, { interpolation: Interpolation.Nearest });

// Smooth result (default)
image.rotate(45, { interpolation: Interpolation.Bilinear });
```

## Enumeration Members

### Bicubic

> **Bicubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bicubic`

***

### Bilinear

> **Bilinear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bilinear`

***

### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Nearest`
