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

### delta

> `readonly` **delta**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delta

***

### total

> `readonly` **total**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
