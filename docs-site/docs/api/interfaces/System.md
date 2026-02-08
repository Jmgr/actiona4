# Interface: System

Defined in: [index.d.ts:5937](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5937)

System information and power/session operations.

```ts
const cpuUsage = await system.cpu.usage();
const memory = await system.memory.usage();

console.log(formatPercent(cpuUsage), formatBytes(memory.used));
```

```ts
const interfaces = await system.network.listInterfaces();
console.log(`interfaces: ${interfaces.length}`);
```

## Properties

### cpu

> `readonly` **cpu**: [`Cpu`](Cpu.md)

Defined in: [index.d.ts:5941](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5941)

Cpu information

***

### hardware

> `readonly` **hardware**: [`Hardware`](Hardware.md)

Defined in: [index.d.ts:5945](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5945)

Hardware information

***

### memory

> `readonly` **memory**: [`Memory`](Memory.md)

Defined in: [index.d.ts:5949](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5949)

Memory information

***

### network

> `readonly` **network**: [`Network`](Network.md)

Defined in: [index.d.ts:5953](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5953)

Network information

***

### os

> `readonly` **os**: [`Os`](Os.md)

Defined in: [index.d.ts:5957](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5957)

Os information

***

### processes

> `readonly` **processes**: [`Processes`](Processes.md)

Defined in: [index.d.ts:5961](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5961)

Processes information

***

### storage

> `readonly` **storage**: [`Storage`](Storage.md)

Defined in: [index.d.ts:5965](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5965)

Storage information

## Methods

### hibernate()

> **hibernate**(): `Promise`\<`void`\>

Defined in: [index.d.ts:5969](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5969)

#### Returns

`Promise`\<`void`\>

***

### logout()

> **logout**(`force?`): `Promise`\<`void`\>

Defined in: [index.d.ts:5968](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5968)

#### Parameters

##### force?

`boolean`

#### Returns

`Promise`\<`void`\>

***

### open()

> **open**(`path`, `withApp?`): `Promise`\<`void`\>

Defined in: [index.d.ts:5971](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5971)

#### Parameters

##### path

`string`

##### withApp?

`string`

#### Returns

`Promise`\<`void`\>

***

### openPath()

> **openPath**(`path`, `withApp?`): `Promise`\<`void`\>

Defined in: [index.d.ts:5972](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5972)

#### Parameters

##### path

`string`

##### withApp?

`string`

#### Returns

`Promise`\<`void`\>

***

### reboot()

> **reboot**(`force?`): `Promise`\<`void`\>

Defined in: [index.d.ts:5967](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5967)

#### Parameters

##### force?

`boolean`

#### Returns

`Promise`\<`void`\>

***

### shutdown()

> **shutdown**(`force?`): `Promise`\<`void`\>

Defined in: [index.d.ts:5966](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5966)

#### Parameters

##### force?

`boolean`

#### Returns

`Promise`\<`void`\>

***

### sleep()

> **sleep**(): `Promise`\<`void`\>

Defined in: [index.d.ts:5970](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5970)

#### Returns

`Promise`\<`void`\>
