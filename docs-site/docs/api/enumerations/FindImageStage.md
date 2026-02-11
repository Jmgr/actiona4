# Enumeration: FindImageStage

Stages of a find image operation.

```ts
const task = source.findImage(template);
for await (const progress of task) {
if (progress.stage === FindImageStage.Matching) {
println(`Matching: ${formatPercent(progress.percent)}`);
}
}
```

## Enumeration Members

### Capturing

> **Capturing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### ComputingResults

> **ComputingResults**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Downscaling

> **Downscaling**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Filtering

> **Filtering**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Finished

> **Finished**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Matching

> **Matching**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Preparing

> **Preparing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)
