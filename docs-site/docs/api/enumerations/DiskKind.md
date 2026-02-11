# Enumeration: DiskKind

Disk kind values.

```ts
const disks = await system.storage.listDisks();
const disk = disks[0];
if (disk && disk.kind === DiskKind.SSD) {
println("SSD");
}
```

Disk kind

## Enumeration Members

### Hdd

> **Hdd**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Hard disk drive

***

### Ssd

> **Ssd**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Solid-state drive

***

### Unknown

> **Unknown**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Unknown drive kind
