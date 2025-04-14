# Interface: Storage

Storage devices and disk usage information.

```ts
const disks = await system.storage.listDisks();
println(disks.length);
```

## Methods

### listDisks()

> <span class="async-badge">async</span> **listDisks**(`options?`: [`ListDisksOptions`](ListDisksOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Disk`](Disk.md)[]\>

Disks

#### Parameters

##### options?

[`ListDisksOptions`](ListDisksOptions.md)

<div class="options-fields">

###### rescan?

> `optional` **rescan?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Rescan

###### Default Value

`true`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Disk`](Disk.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this storage.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
