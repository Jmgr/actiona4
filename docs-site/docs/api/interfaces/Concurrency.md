# Interface: Concurrency

Utilities for concurrent operations.

```ts
// Race two promises, resolving with whichever finishes first, cancelling the other.
// Note that this is different from `Promises.race`, which doesn't cancel any promise.
const result = await concurrency.race([sleep("100ms"), sleep("1s")]);
```

## Methods

### race()

> <span class="async-badge">async</span> **race**\<`T`\>(`promises`: [`Iterable`](https://www.typescriptlang.org/docs/handbook/iterators-and-generators.html#iterable-interface)\<`T` \| `PromiseLike`\<`T`\>\>): [`Task`](../type-aliases/Task.md)\<[`Awaited`](https://www.typescriptlang.org/docs/handbook/utility-types.html#awaitedtype)\<`T`\>\>

Races multiple promises, returning the result of the first one to settle.
Losing tasks will be cancelled automatically.

```ts
// Resolve with the first successful result.
const result = await concurrency.race([
  sleep("200ms").then(() => "fast"),
  sleep("1s").then(() => "slow"),
]);
// result === "fast"
```

```ts
// Use race to implement a timeout.
const result = await concurrency.race([
  fetchData(),
  sleep("5s").then(() => { throw new Error("Timeout"); })
]);
```

```ts
// Rejections also win the race.
// Here the error is thrown quickly and the slower task is cancelled.
try {
  await concurrency.race([
    sleep("50ms").then(() => { throw new Error("Failed quickly"); }),
    sleep("2s"),
  ]);
} catch (e) {
  console.println(e); // Error: Failed quickly
}
```

```ts
// You can cancel the race task itself.
const t = concurrency.race([
  sleep("5s"),
  sleep("8s"),
]);
t.cancel();
await t; // throws "Error: Cancelled"
```

```ts
// Empty or non-promise-only inputs resolve to undefined.
const a = await concurrency.race([]);
const b = await concurrency.race([1, "text", null]);
// a === undefined, b === undefined
```

#### Type Parameters

##### T

`T`

#### Parameters

##### promises

[`Iterable`](https://www.typescriptlang.org/docs/handbook/iterators-and-generators.html#iterable-interface)\<`T` \| `PromiseLike`\<`T`\>\>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Awaited`](https://www.typescriptlang.org/docs/handbook/utility-types.html#awaitedtype)\<`T`\>\>
