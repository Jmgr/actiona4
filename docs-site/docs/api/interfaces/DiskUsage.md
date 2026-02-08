# Interface: DiskUsage

Defined in: [index.d.ts:6496](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6496)

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

> `readonly` **read**: `Readonly`\<[`IoStats`](IoStats.md)\>

Defined in: [index.d.ts:6504](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6504)

Read

***

### written

> `readonly` **written**: `Readonly`\<[`IoStats`](IoStats.md)\>

Defined in: [index.d.ts:6500](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6500)

Written

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6505](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6505)

#### Returns

`string`
