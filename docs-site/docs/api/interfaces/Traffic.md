# Interface: Traffic

Defined in: [index.d.ts:6092](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6092)

Traffic statistics.

```ts
const interfaces = await system.network.listInterfaces();
const iface = interfaces[0];
if (iface) {
console.log(
formatBytes(iface.inbound.total.data),
formatBytes(iface.inbound.delta.data),
);
}
```

## Properties

### delta

> `readonly` **delta**: `Readonly`\<[`Counters`](Counters.md)\>

Defined in: [index.d.ts:6100](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6100)

Delta

***

### total

> `readonly` **total**: `Readonly`\<[`Counters`](Counters.md)\>

Defined in: [index.d.ts:6096](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6096)

Total

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6101](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6101)

#### Returns

`string`
