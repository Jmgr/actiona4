# Interface: Hardware

Hardware information.

```ts
const hw = system.hardware;
const board = hw.motherboard;
const components = await hw.listComponents();

console.log(hw.vendorName, board.name, components.length);
```

## Properties

### family?

> `readonly` `optional` **family**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Family

***

### motherboard

> `readonly` **motherboard**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Motherboard`](Motherboard.md)\>

Motherboard

***

### name?

> `readonly` `optional` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### serialNumber?

> `readonly` `optional` **serialNumber**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Serial number

***

### stockKeepingUnit?

> `readonly` `optional` **stockKeepingUnit**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Stock keeping unit

***

### uuid?

> `readonly` `optional` **uuid**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Uuid

***

### vendorName?

> `readonly` `optional` **vendorName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Vendor name

***

### version?

> `readonly` `optional` **version**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Version

## Methods

### listComponents()

> **listComponents**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Component`](Component.md)[]\>

Hardware components

#### Parameters

##### options?

[`ListComponentsOptions`](ListComponentsOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Component`](Component.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
