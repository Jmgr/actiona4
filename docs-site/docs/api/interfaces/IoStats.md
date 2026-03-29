# Interface: IoStats

Disk I/O statistics (bytes).

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk) {
  println(
    formatBytes(disk.usage.read.total),
    formatBytes(disk.usage.written.delta),
  );
}
```

## Properties

### total

> `readonly` **total**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total

***

### delta

> `readonly` **delta**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delta

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of these I/O stats.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
