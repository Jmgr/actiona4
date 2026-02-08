# Interface: Cpu

Defined in: [index.d.ts:5677](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5677)

CPU metrics and topology.

```ts
const globalUsage = await system.cpu.usage();
const core0Usage = await system.cpu.coreUsage(0);
const freqs = await system.cpu.frequencies();

console.log(
system.cpu.logicalCoreCount,
formatPercent(globalUsage),
formatPercent(core0Usage),
formatFrequency(freqs[0]),
);
```

## Properties

### architecture

> `readonly` **architecture**: `string`

Defined in: [index.d.ts:5689](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5689)

Architecture

***

### logicalCoreCount

> `readonly` **logicalCoreCount**: `number`

Defined in: [index.d.ts:5681](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5681)

Logical core count

***

### physicalCoreCount?

> `readonly` `optional` **physicalCoreCount**: `number`

Defined in: [index.d.ts:5685](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5685)

Physical core count

## Methods

### coreUsage()

> **coreUsage**(`logicalCoreIndex`): `Promise`\<`number`\>

Defined in: [index.d.ts:5691](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5691)

#### Parameters

##### logicalCoreIndex

`number`

#### Returns

`Promise`\<`number`\>

***

### frequencies()

> **frequencies**(): `Promise`\<readonly `number`[]\>

Defined in: [index.d.ts:5692](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5692)

#### Returns

`Promise`\<readonly `number`[]\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5693](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5693)

#### Returns

`string`

***

### usage()

> **usage**(): `Promise`\<`number`\>

Defined in: [index.d.ts:5690](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5690)

#### Returns

`Promise`\<`number`\>
