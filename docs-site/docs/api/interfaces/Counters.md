# Interface: Counters

Defined in: [index.d.ts:6062](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6062)

Byte/packet/error counters.

```ts
const interfaces = await system.network.listInterfaces();
const iface = interfaces[0];
if (iface) {
const counters = iface.inbound.total;
console.log(formatBytes(counters.data), counters.packets, counters.errors);
}
```

## Properties

### data

> `readonly` **data**: `number`

Defined in: [index.d.ts:6066](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6066)

Data

***

### errors

> `readonly` **errors**: `number`

Defined in: [index.d.ts:6074](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6074)

Errors

***

### packets

> `readonly` **packets**: `number`

Defined in: [index.d.ts:6070](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6070)

Packets

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6075](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6075)

#### Returns

`string`
