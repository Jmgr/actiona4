# Interface: NetworkInterface

Defined in: [index.d.ts:6022](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6022)

A network interface.

```ts
const interfaces = await system.network.listInterfaces();
const iface = interfaces[0];
if (iface) {
console.log(iface.name, iface.mtu, iface.macAddress);
}
```

## Properties

### inbound

> `readonly` **inbound**: `Readonly`\<[`Traffic`](Traffic.md)\>

Defined in: [index.d.ts:6030](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6030)

Inbound

***

### macAddress?

> `readonly` `optional` **macAddress**: `string`

Defined in: [index.d.ts:6042](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6042)

MAC address

***

### mtu

> `readonly` **mtu**: `number`

Defined in: [index.d.ts:6038](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6038)

MTU

***

### name

> `readonly` **name**: `string`

Defined in: [index.d.ts:6026](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6026)

Name

***

### outbound

> `readonly` **outbound**: `Readonly`\<[`Traffic`](Traffic.md)\>

Defined in: [index.d.ts:6034](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6034)

Outbound

***

### subnets

> `readonly` **subnets**: readonly `string`[]

Defined in: [index.d.ts:6046](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6046)

Subnets

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6047](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6047)

#### Returns

`string`
