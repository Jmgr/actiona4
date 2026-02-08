# Interface: MemoryUsage

Defined in: [index.d.ts:5865](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5865)

A memory usage snapshot.

```ts
const usage = await system.memory.usage();
console.log(
formatBytes(usage.used),
formatBytes(usage.free),
formatBytes(usage.available),
formatBytes(usage.total),
);
```

## Properties

### available

> `readonly` **available**: `number`

Defined in: [index.d.ts:5877](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5877)

Available

***

### free

> `readonly` **free**: `number`

Defined in: [index.d.ts:5873](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5873)

Free

***

### total

> `readonly` **total**: `number`

Defined in: [index.d.ts:5881](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5881)

Total

***

### used

> `readonly` **used**: `number`

Defined in: [index.d.ts:5869](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5869)

Used

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5882](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5882)

#### Returns

`string`
