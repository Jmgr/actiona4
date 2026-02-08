# Interface: IoStats

Defined in: [index.d.ts:6470](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6470)

Disk I/O statistics (bytes).

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk) {
console.log(
formatBytes(disk.usage.read.total),
formatBytes(disk.usage.written.delta),
);
}
```

## Properties

### delta

> `readonly` **delta**: `number`

Defined in: [index.d.ts:6478](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6478)

Delta

***

### total

> `readonly` **total**: `number`

Defined in: [index.d.ts:6474](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6474)

Total

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6479](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6479)

#### Returns

`string`
