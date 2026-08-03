# Interface: FindImageProgress

Progress of a find image operation.

Received by iterating over the async iterator returned by `find` or `findAll`.

```ts
const task = source.find(template);
for await (const progress of task) {
  println(`${progress.step}: ${formatPercent(progress.progress * 100)}`);
  if (progress.finished) break;
}
const result = await task;
```

## Properties

### step

> `readonly` **step**: [`FindImageStep`](../enumerations/FindImageStep.md)

The current step of the find image operation.

***

### progress

> `readonly` **progress**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Completion of the whole search, from 0 to 1.

***

### stepProgress

> `readonly` **stepProgress**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Completion of the current step, from 0 to 1. Steps that cannot measure
themselves report 0 when they start and 1 when they end.

***

### finished

> `readonly` **finished**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the operation has finished.

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this image search progress.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
