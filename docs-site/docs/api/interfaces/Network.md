# Interface: Network

Network information and interfaces.

```ts
console.log(system.network.hostname);
const interfaces = await system.network.listInterfaces();
console.log(interfaces.length);
```

## Properties

### hostname?

> `readonly` `optional` **hostname**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Host name

## Methods

### listInterfaces()

> **listInterfaces**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`NetworkInterface`](NetworkInterface.md)[]\>

Interfaces

#### Parameters

##### options?

[`ListInterfacesOptions`](ListInterfacesOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`NetworkInterface`](NetworkInterface.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
