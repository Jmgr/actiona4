# Interface: Memory

Memory metrics.

```ts
const usage = await system.memory.usage();
const swap = await system.memory.swapUsage();

println(formatBytes(usage.used), formatBytes(swap.used));
```

## Properties

### cgroupLimits?

> `readonly` `optional` **cgroupLimits**: [`CGroupLimits`](CGroupLimits.md)

CGroup limits

#### Platform

only works on Linux

## Methods

### swapUsage()

> <span class="async-badge">async</span> **swapUsage**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>

Swap usage

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### usage()

> <span class="async-badge">async</span> **usage**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>

Memory usage

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>
