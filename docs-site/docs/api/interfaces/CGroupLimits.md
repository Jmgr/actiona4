# Interface: CGroupLimits

CGroup memory and swap limits.

```ts
const limits = system.memory.cgroupLimits;
if (limits) {
println(
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

> `readonly` **freeMemory**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Free memory

***

### freeSwap

> `readonly` **freeSwap**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Free swap

***

### rss

> `readonly` **rss**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

RSS

***

### totalMemory

> `readonly` **totalMemory**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total memory

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
