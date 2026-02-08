# Enumeration: Interpolation

Defined in: [index.d.ts:142](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L142)

Interpolation algorithms used for image rotations.

```ts
// Fast but lower quality
image.rotate(45, { interpolation: Interpolation.Nearest });

// Smooth result (default)
image.rotate(45, { interpolation: Interpolation.Bilinear });
```

## Enumeration Members

### Bicubic

> **Bicubic**: `number`

Defined in: [index.d.ts:147](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L147)

***

### Bilinear

> **Bilinear**: `number`

Defined in: [index.d.ts:145](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L145)

***

### Nearest

> **Nearest**: `number`

Defined in: [index.d.ts:143](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L143)
