# Interface: Disk

Defined in: [index.d.ts:6416](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6416)

A disk device.

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk) {
console.log(
disk.name,
disk.kind,
disk.mountPoint,
formatBytes(disk.totalSpace),
formatBytes(disk.availableSpace),
);
}
```

## Properties

### availableSpace

> `readonly` **availableSpace**: `number`

Defined in: [index.d.ts:6440](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6440)

Available space

***

### fileSystem?

> `readonly` `optional` **fileSystem**: `string`

Defined in: [index.d.ts:6428](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6428)

File system

***

### isReadOnly

> `readonly` **isReadOnly**: `boolean`

Defined in: [index.d.ts:6448](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6448)

Is read-only

***

### isRemovable

> `readonly` **isRemovable**: `boolean`

Defined in: [index.d.ts:6444](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6444)

Is removable

***

### kind

> `readonly` **kind**: [`DiskKind`](../enumerations/DiskKind.md)

Defined in: [index.d.ts:6420](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6420)

Kind

***

### mountPoint

> `readonly` **mountPoint**: `string`

Defined in: [index.d.ts:6432](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6432)

Mount point

***

### name?

> `readonly` `optional` **name**: `string`

Defined in: [index.d.ts:6424](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6424)

Name

***

### totalSpace

> `readonly` **totalSpace**: `number`

Defined in: [index.d.ts:6436](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6436)

Total space

***

### usage

> `readonly` **usage**: `Readonly`\<[`DiskUsage`](DiskUsage.md)\>

Defined in: [index.d.ts:6452](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6452)

Usage

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6453](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6453)

#### Returns

`string`
