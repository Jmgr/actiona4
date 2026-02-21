# Interface: FindImageProgress

Progress of a find image operation.

Received by iterating over the async iterator returned by `findImage` or `findImageAll`.

```ts
const task = source.findImage(template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
  if (progress.finished) break;
}
const result = await task;
```

## Properties

### finished

> `readonly` **finished**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the operation has finished.

***

### percent

> `readonly` **percent**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Completion percentage (0-100).

***

### stage

> `readonly` **stage**: [`FindImageStage`](../enumerations/FindImageStage.md)

The current stage of the find image operation.

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
