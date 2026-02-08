# Interface: Storage

Defined in: [index.d.ts:6380](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6380)

Storage devices and disk usage information.

```ts
const disks = await system.storage.listDisks();
console.log(disks.length);
```

## Methods

### listDisks()

> **listDisks**(`options?`): `Promise`\<readonly [`Disk`](Disk.md)[]\>

Defined in: [index.d.ts:6384](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6384)

Disks

#### Parameters

##### options?

[`ListDisksOptions`](ListDisksOptions.md)

#### Returns

`Promise`\<readonly [`Disk`](Disk.md)[]\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6385](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6385)

#### Returns

`string`
