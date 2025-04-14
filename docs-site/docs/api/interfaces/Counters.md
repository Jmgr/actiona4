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

### packets

> `readonly` **packets**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Packets

***

### errors

> `readonly` **errors**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Errors

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of these counters.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
