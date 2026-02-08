# Interface: MemoryUsage

A memory usage snapshot.

```ts
const usage = await system.memory.usage();
console.log(
formatBytes(usage.used),
formatBytes(usage.free),
formatBytes(usage.available),
formatBytes(usage.total),
);
```

## Properties

### available

> `readonly` **available**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Available

***

### free

> `readonly` **free**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Free

***

### total

> `readonly` **total**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total

***

### used

> `readonly` **used**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Used

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
