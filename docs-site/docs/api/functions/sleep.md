# Function: sleep()

> **sleep**(`duration`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Pauses the execution for the given duration.

```ts
// Wait 500 milliseconds
await sleep(500);

// Wait 1 second
await sleep("1s");

// Wait 1 hour
await sleep("1h");
```

## Parameters

### duration

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) | [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

## Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
