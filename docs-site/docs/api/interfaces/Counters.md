# Interface: Counters

Byte/packet/error counters.

```ts
const interfaces = await system.network.listInterfaces();
const iface = interfaces[0];
if (iface) {
  const counters = iface.inbound.total;
  println(formatBytes(counters.data), counters.packets, counters.errors);
}
```

## Properties

### data

> `readonly` **data**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Data

***

### errors

> `readonly` **errors**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Errors

***

### packets

> `readonly` **packets**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Packets

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
