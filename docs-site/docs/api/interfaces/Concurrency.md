# Interface: Concurrency

Utilities for concurrent operations.

```ts
// Race two promises, resolving with whichever finishes first, cancelling the other.
// Note that this is different from `Promises.race`, which doesn't cancel any promise.
const result = await Concurrency.race([sleep(100), sleep(1000)]);
```

## Methods

### race()

> **race**\<`T`\>(`promises`): [`Task`](../type-aliases/Task.md)\<[`Awaited`](https://www.typescriptlang.org/docs/handbook/utility-types.html#awaitedtype)\<`T`\>\>

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

[`Iterable`](https://www.typescriptlang.org/docs/handbook/iterators-and-generators.html#iterable-interface)\<`T` \| `PromiseLike`\<`T`\>\>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Awaited`](https://www.typescriptlang.org/docs/handbook/utility-types.html#awaitedtype)\<`T`\>\>
