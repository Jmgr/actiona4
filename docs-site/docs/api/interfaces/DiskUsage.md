# Interface: DiskUsage

Read/write usage for a disk.

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk) {
console.log(
formatBytes(disk.usage.read.total),
formatBytes(disk.usage.written.total),
);
}
```

## Properties

### read

> `readonly` **read**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`IoStats`](IoStats.md)\>

Read

***

### written

> `readonly` **written**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`IoStats`](IoStats.md)\>

Written

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
