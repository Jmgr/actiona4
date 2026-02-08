# Interface: Hardware

Defined in: [index.d.ts:5707](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5707)

Hardware information.

```ts
const hw = system.hardware;
const board = hw.motherboard;
const components = await hw.listComponents();

console.log(hw.vendorName, board.name, components.length);
```

## Properties

### family?

> `readonly` `optional` **family**: `string`

Defined in: [index.d.ts:5715](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5715)

Family

***

### motherboard

> `readonly` **motherboard**: `Readonly`\<[`Motherboard`](Motherboard.md)\>

Defined in: [index.d.ts:5739](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5739)

Motherboard

***

### name?

> `readonly` `optional` **name**: `string`

Defined in: [index.d.ts:5711](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5711)

Name

***

### serialNumber?

> `readonly` `optional` **serialNumber**: `string`

Defined in: [index.d.ts:5719](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5719)

Serial number

***

### stockKeepingUnit?

> `readonly` `optional` **stockKeepingUnit**: `string`

Defined in: [index.d.ts:5723](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5723)

Stock keeping unit

***

### uuid?

> `readonly` `optional` **uuid**: `string`

Defined in: [index.d.ts:5731](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5731)

Uuid

***

### vendorName?

> `readonly` `optional` **vendorName**: `string`

Defined in: [index.d.ts:5735](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5735)

Vendor name

***

### version?

> `readonly` `optional` **version**: `string`

Defined in: [index.d.ts:5727](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5727)

Version

## Methods

### listComponents()

> **listComponents**(`options?`): `Promise`\<readonly [`Component`](Component.md)[]\>

Defined in: [index.d.ts:5743](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5743)

Hardware components

#### Parameters

##### options?

[`ListComponentsOptions`](ListComponentsOptions.md)

#### Returns

`Promise`\<readonly [`Component`](Component.md)[]\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5744](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5744)

#### Returns

`string`
