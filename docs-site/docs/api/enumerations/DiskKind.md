# Enumeration: DiskKind

Defined in: [index.d.ts:1781](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1781)

Disk kind values.

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk && disk.kind === DiskKind.SSD) {
console.log("SSD");
}
```

Disk kind

## Enumeration Members

### Hdd

> **Hdd**: `number`

Defined in: [index.d.ts:1785](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1785)

Hard disk drive

***

### Ssd

> **Ssd**: `number`

Defined in: [index.d.ts:1790](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1790)

Solid-state drive

***

### Unknown

> **Unknown**: `number`

Defined in: [index.d.ts:1795](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1795)

Unknown drive kind
