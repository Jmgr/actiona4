# Enumeration: FindImageStep


Steps of a find image operation.

```ts
const task = source.find(template);
for await (const progress of task) {
  if (progress.step === FindImageStep.Matching) {
    println(`Matching: ${formatPercent(progress.progress * 100)}`);
  }
}
```

## Enumeration Members

### Capturing

> **Capturing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.Capturing`

***

### Preparing

> **Preparing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.Preparing`

***

### Downscaling

> **Downscaling**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.Downscaling`

***

### Matching

> **Matching**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.Matching`

***

### Filtering

> **Filtering**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.Filtering`

***

### ComputingResults

> **ComputingResults**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.ComputingResults`

***

### Finished

> **Finished**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FindImageStep.Finished`
