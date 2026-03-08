# Class: PlayProgress

Progress of a `macros.play()` operation.

Received by iterating over the async iterator returned by `play`.

```ts
const task = macros.play(macro);
for await (const progress of task) {
    console.println(`${Math.round(progress.ratio() * 100)}%`);
    if (progress.finished()) break;
}
await task;
```

## Methods

### eventsDone()

> **eventsDone**(): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Number of events replayed so far.

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### finished()

> **finished**(): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether all events have been replayed.

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### ratio()

> **ratio**(): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Replay ratio, in the range `[0, 1]`.

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### totalEvents()

> **totalEvents**(): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total number of events to replay.

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)
