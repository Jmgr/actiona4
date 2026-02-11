# Interface: Storage

Storage devices and disk usage information.

```ts
const disks = await system.storage.listDisks();
println(disks.length);
```

## Methods

### listDisks()

> **listDisks**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Disk`](Disk.md)[]\>

Disks

#### Parameters

##### options?

[`ListDisksOptions`](ListDisksOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Disk`](Disk.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
