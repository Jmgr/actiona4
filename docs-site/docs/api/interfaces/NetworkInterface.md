# Interface: NetworkInterface

A network interface.

```ts
const interfaces = await system.network.listInterfaces();
const iface = interfaces[0];
if (iface) {
  println(iface.name, iface.mtu, iface.macAddress);
}
```

## Properties

### name

> `readonly` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### inbound

> `readonly` **inbound**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Traffic`](Traffic.md)\>

Inbound

***

### outbound

> `readonly` **outbound**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Traffic`](Traffic.md)\>

Outbound

***

### mtu

> `readonly` **mtu**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

MTU

***

### macAddress?

> `readonly` `optional` **macAddress?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

MAC address

***

### subnets

> `readonly` **subnets**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Subnets

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this network interface.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
