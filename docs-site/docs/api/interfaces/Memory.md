# Interface: Memory

Defined in: [index.d.ts:5835](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5835)

Memory metrics.

```ts
const usage = await system.memory.usage();
const swap = await system.memory.swapUsage();

console.log(formatBytes(usage.used), formatBytes(swap.used));
```

## Properties

### cgroupLimits?

> `readonly` `optional` **cgroupLimits**: [`CGroupLimits`](CGroupLimits.md)

Defined in: [index.d.ts:5840](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5840)

CGroup limits

#### Platform

only works on Linux

## Methods

### swapUsage()

> **swapUsage**(): `Promise`\<[`MemoryUsage`](MemoryUsage.md)\>

Defined in: [index.d.ts:5848](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5848)

Swap usage

#### Returns

`Promise`\<[`MemoryUsage`](MemoryUsage.md)\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5849](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5849)

#### Returns

`string`

***

### usage()

> **usage**(): `Promise`\<[`MemoryUsage`](MemoryUsage.md)\>

Defined in: [index.d.ts:5844](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5844)

Memory usage

#### Returns

`Promise`\<[`MemoryUsage`](MemoryUsage.md)\>
