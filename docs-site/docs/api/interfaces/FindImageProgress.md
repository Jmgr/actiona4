# Interface: FindImageProgress

Defined in: [index.d.ts:4051](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4051)

Progress of a find image operation.

Received by iterating over the async iterator returned by `findImage` or `findImageAll`.

```ts
const task = source.findImage(template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
if (progress.finished) break;
}
const result = await task;
```

## Properties

### finished

> `readonly` **finished**: `boolean`

Defined in: [index.d.ts:4063](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4063)

Whether the operation has finished.

***

### percent

> `readonly` **percent**: `number`

Defined in: [index.d.ts:4059](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4059)

Completion percentage (0-100).

***

### stage

> `readonly` **stage**: [`FindImageStage`](../enumerations/FindImageStage.md)

Defined in: [index.d.ts:4055](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4055)

The current stage of the find image operation.
