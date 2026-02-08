# Interface: Concurrency

Defined in: [index.d.ts:4622](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4622)

Utilities for concurrent operations.

```ts
// Race two promises, resolving with whichever finishes first, cancelling the other.
// Note that this is different from `Promises.race`, which doesn't cancel any promise.
const result = await Concurrency.race([sleep(100), sleep(1000)]);
```

## Methods

### race()

> **race**\<`T`\>(`promises`): [`Task`](../type-aliases/Task.md)\<`Awaited`\<`T`\>\>

Defined in: [index.d.ts:4635](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4635)

Races multiple promises, returning the result of the first one to settle.
Losing tasks will be cancelled automatically.

```ts
// Use race to implement a timeout
const result = await Concurrency.race([
fetchData(),
sleep(5000).then(() => { throw new Error("Timeout"); })
]);
```

#### Type Parameters

##### T

`T`

#### Parameters

##### promises

`Iterable`\<`T` \| `PromiseLike`\<`T`\>\>

#### Returns

[`Task`](../type-aliases/Task.md)\<`Awaited`\<`T`\>\>
