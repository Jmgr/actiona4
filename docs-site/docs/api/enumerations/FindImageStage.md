# Enumeration: FindImageStage


Stages of a find image operation.

```ts
const task = source.find(template);
for await (const progress of task) {
  if (progress.stage === FindImageStage.Matching) {
    println(`Matching: ${formatPercent(progress.percent)}`);
  }
}
```

## Enumeration Members

### Capturing

> **Capturing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.Capturing`

***

### ComputingResults

> **ComputingResults**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.ComputingResults`

***

### Downscaling

> **Downscaling**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.Downscaling`

***

### Filtering

> **Filtering**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.Filtering`

***

### Finished

> **Finished**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.Finished`

***

### Matching

> **Matching**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.Matching`

***

### Preparing

> **Preparing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStage.Preparing`
