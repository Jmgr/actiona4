# Type Alias: Task\<Result\>

> **Task**\<`Result`\> = [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Result`\> & [`object`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Object)

A cancellable promise.

```ts
const task = sleep("5s");
task.cancel(); // cancel early
await task;    // or await it
```

## Type Declaration

### cancel()

> **cancel**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

## Type Parameters

### Result

`Result`
