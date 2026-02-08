# Enumeration: FindImageStage

Defined in: [index.d.ts:196](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L196)

Stages of a find image operation.

```ts
const task = source.findImage(template);
for await (const progress of task) {
if (progress.stage === FindImageStage.Matching) {
console.log(`Matching: ${formatPercent(progress.percent)}`);
}
}
```

## Enumeration Members

### Capturing

> **Capturing**: `number`

Defined in: [index.d.ts:197](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L197)

***

### ComputingResults

> **ComputingResults**: `number`

Defined in: [index.d.ts:207](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L207)

***

### Downscaling

> **Downscaling**: `number`

Defined in: [index.d.ts:201](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L201)

***

### Filtering

> **Filtering**: `number`

Defined in: [index.d.ts:205](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L205)

***

### Finished

> **Finished**: `number`

Defined in: [index.d.ts:209](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L209)

***

### Matching

> **Matching**: `number`

Defined in: [index.d.ts:203](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L203)

***

### Preparing

> **Preparing**: `number`

Defined in: [index.d.ts:199](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L199)
