# Enumeration: ResizeFilter

Defined in: [index.d.ts:119](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L119)

Resize filter algorithms.

```ts
// Use nearest-neighbor for pixel art (no smoothing)
image.resize(64, 64, { filter: ResizeFilter.Nearest });

// Use Lanczos3 for high-quality downscaling
image.resize(200, 150, { filter: ResizeFilter.Lanczos3 });
```

## Enumeration Members

### Cubic

> **Cubic**: `number`

Defined in: [index.d.ts:124](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L124)

***

### Gaussian

> **Gaussian**: `number`

Defined in: [index.d.ts:126](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L126)

***

### Lanczos3

> **Lanczos3**: `number`

Defined in: [index.d.ts:128](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L128)

***

### Linear

> **Linear**: `number`

Defined in: [index.d.ts:122](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L122)

***

### Nearest

> **Nearest**: `number`

Defined in: [index.d.ts:120](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L120)
