# Interface: Cpu

CPU metrics and topology.

```ts
const globalUsage = await system.cpu.usage();
const core0Usage = await system.cpu.coreUsage(0);
const freqs = await system.cpu.frequencies();

println(
  system.cpu.logicalCoreCount,
  formatPercent(globalUsage),
  formatPercent(core0Usage),
  formatFrequency(freqs[0]),
);
```

## Properties

### architecture

> `readonly` **architecture**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Architecture

***

### logicalCoreCount

> `readonly` **logicalCoreCount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Logical core count

***

### physicalCoreCount?

> `readonly` `optional` **physicalCoreCount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Physical core count

## Methods

### coreUsage()

> **coreUsage**(`logicalCoreIndex`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

#### Parameters

##### logicalCoreIndex

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

***

### frequencies()

> **frequencies**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)[]\>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### usage()

> **usage**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>
