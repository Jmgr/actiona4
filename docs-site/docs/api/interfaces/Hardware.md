# Interface: Hardware

Hardware information.

```ts
const hw = system.hardware;
const board = hw.motherboard;
const temperatureSensors = await hw.listTemperatureSensors();

println(hw.vendorName, board.name, temperatureSensors.length);
```

## Properties

### family?

> `readonly` `optional` **family?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Family

***

### motherboard

> `readonly` **motherboard**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Motherboard`](Motherboard.md)\>

Motherboard

***

### name?

> `readonly` `optional` **name?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### serialNumber?

> `readonly` `optional` **serialNumber?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Serial number

***

### stockKeepingUnit?

> `readonly` `optional` **stockKeepingUnit?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Stock keeping unit

***

### uuid?

> `readonly` `optional` **uuid?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Uuid

***

### vendorName?

> `readonly` `optional` **vendorName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Vendor name

***

### version?

> `readonly` `optional` **version?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Version

## Methods

### listTemperatureSensors()

> <span class="async-badge">async</span> **listTemperatureSensors**(`options?`: [`ListTemperatureSensorsOptions`](ListTemperatureSensorsOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`TemperatureSensor`](TemperatureSensor.md)[]\>

Hardware temperature sensors

#### Parameters

##### options?

[`ListTemperatureSensorsOptions`](ListTemperatureSensorsOptions.md)

<div class="options-fields">

###### rescan?

> `optional` **rescan?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Rescan

###### Default Value

`true`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`TemperatureSensor`](TemperatureSensor.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this hardware.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
