# Interface: CGroupLimits

Defined in: [index.d.ts:5902](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5902)

CGroup memory and swap limits.

```ts
const limits = system.memory.cgroupLimits;
if (limits) {
console.log(
formatBytes(limits.totalMemory),
formatBytes(limits.freeMemory),
formatBytes(limits.freeSwap),
);
}
```

CGroup limits

## Platform

only works on Linux

## Properties

### freeMemory

> `readonly` **freeMemory**: `number`

Defined in: [index.d.ts:5910](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5910)

Free memory

***

### freeSwap

> `readonly` **freeSwap**: `number`

Defined in: [index.d.ts:5914](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5914)

Free swap

***

### rss

> `readonly` **rss**: `number`

Defined in: [index.d.ts:5918](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5918)

RSS

***

### totalMemory

> `readonly` **totalMemory**: `number`

Defined in: [index.d.ts:5906](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5906)

Total memory

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5919](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5919)

#### Returns

`string`
