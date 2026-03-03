# Interface: Network

Network information and interfaces.

```ts
println(system.network.hostname);
const interfaces = await system.network.listInterfaces();
println(interfaces.length);
```

## Properties

### hostname?

> `readonly` `optional` **hostname**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Host name

## Methods

### listInterfaces()

> <span class="async-badge">async</span> **listInterfaces**(`options?`: [`ListInterfacesOptions`](ListInterfacesOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`NetworkInterface`](NetworkInterface.md)[]\>

Interfaces

#### Parameters

##### options?

[`ListInterfacesOptions`](ListInterfacesOptions.md)

<div class="options-fields">

###### rescan?

> `optional` **rescan**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Rescan

###### Default Value

`true`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`NetworkInterface`](NetworkInterface.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
