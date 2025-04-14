# Function: sleep()

> <span class="async-badge">async</span> **sleep**(`duration`: [`DurationLike`](../type-aliases/DurationLike.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Pauses the execution for the given duration.

```ts
// Wait 500 milliseconds
await sleep(500);

// Wait 1 second
await sleep("1s");

// Wait 1 hour
await sleep("1h");
```
Numeric values are interpreted as milliseconds.

## Parameters

### duration

[`DurationLike`](../type-aliases/DurationLike.md)

## Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
