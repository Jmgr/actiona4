# Interface: Memory

Memory metrics.

```ts
const usage = await system.memory.usage();
const swap = await system.memory.swapUsage();

println(formatBytes(usage.used), formatBytes(swap.used));
```

## Properties

### cgroupLimits?

> `readonly` `optional` **cgroupLimits?**: [`CGroupLimits`](CGroupLimits.md)

CGroup limits

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

## Methods

### swapUsage()

> <span class="async-badge">async</span> **swapUsage**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>

Swap usage

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this memory.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### usage()

> <span class="async-badge">async</span> **usage**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>

Memory usage

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`MemoryUsage`](MemoryUsage.md)\>
