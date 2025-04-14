# Type Alias: ProgressTask\<Result, Progress\>

> **ProgressTask**\<`Result`, `Progress`\> = [`Task`](Task.md)\<`Result`\> & [`object`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Object)

A cancellable promise that also emits progress updates.

```ts
const task = source.find(template);
for await (const progress of task) {
  console.println(`Stage: ${progress.stage}`);
}
const match = await task;
```

## Type Declaration

### \[asyncIterator\]()

> **\[asyncIterator\]**(): [`AsyncIterator`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/AsyncIterator)\<`Progress`\>

#### Returns

[`AsyncIterator`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/AsyncIterator)\<`Progress`\>

## Type Parameters

### Result

`Result`

### Progress

`Progress`
