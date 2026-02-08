# Interface: Network

Defined in: [index.d.ts:5988](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5988)

Network information and interfaces.

```ts
console.log(system.network.hostname);
const interfaces = await system.network.listInterfaces();
console.log(interfaces.length);
```

## Properties

### hostname?

> `readonly` `optional` **hostname**: `string`

Defined in: [index.d.ts:5992](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5992)

Host name

## Methods

### listInterfaces()

> **listInterfaces**(`options?`): `Promise`\<readonly [`NetworkInterface`](NetworkInterface.md)[]\>

Defined in: [index.d.ts:5996](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5996)

Interfaces

#### Parameters

##### options?

[`ListInterfacesOptions`](ListInterfacesOptions.md)

#### Returns

`Promise`\<readonly [`NetworkInterface`](NetworkInterface.md)[]\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5997](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5997)

#### Returns

`string`
