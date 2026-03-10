# Interface: Disk

A disk device.

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk) {
  println(
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

> `readonly` **availableSpace**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Available space

***

### fileSystem?

> `readonly` `optional` **fileSystem**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

File system

***

### isReadOnly

> `readonly` **isReadOnly**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Is read-only

***

### isRemovable

> `readonly` **isRemovable**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Is removable

***

### kind

> `readonly` **kind**: [`DiskKind`](../enumerations/DiskKind.md)

Kind

***

### mountPoint

> `readonly` **mountPoint**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Mount point

***

### name?

> `readonly` `optional` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### totalSpace

> `readonly` **totalSpace**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total space

***

### usage

> `readonly` **usage**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DiskUsage`](DiskUsage.md)\>

Usage

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this disk.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
