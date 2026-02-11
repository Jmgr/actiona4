# Interface: Traffic

Traffic statistics.

```ts
const interfaces = await system.network.listInterfaces();
const iface = interfaces[0];
if (iface) {
println(
formatBytes(iface.inbound.total.data),
formatBytes(iface.inbound.delta.data),
);
}
```

## Properties

### delta

> `readonly` **delta**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Counters`](Counters.md)\>

Delta

***

### total

> `readonly` **total**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Counters`](Counters.md)\>

Total

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
