# Interface: System

System information and power/session operations.

```ts
const cpuUsage = await system.cpu.usage();
const memory = await system.memory.usage();

println(formatPercent(cpuUsage), formatBytes(memory.used));
```

```ts
const interfaces = await system.network.listInterfaces();
println(`interfaces: ${interfaces.length}`);
```

## Properties

### cpu

> `readonly` **cpu**: [`Cpu`](Cpu.md)

Cpu information

***

### hardware

> `readonly` **hardware**: [`Hardware`](Hardware.md)

Hardware information

***

### memory

> `readonly` **memory**: [`Memory`](Memory.md)

Memory information

***

### network

> `readonly` **network**: [`Network`](Network.md)

Network information

***

### os

> `readonly` **os**: [`Os`](Os.md)

Os information

***

### processes

> `readonly` **processes**: [`Processes`](Processes.md)

Processes information

***

### storage

> `readonly` **storage**: [`Storage`](Storage.md)

Storage information

## Methods

### shutdown()

> **shutdown**(`force?`: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Parameters

##### force?

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### reboot()

> **reboot**(`force?`: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Parameters

##### force?

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### logout()

> **logout**(`force?`: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Parameters

##### force?

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### hibernate()

> **hibernate**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### sleep()

> **sleep**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### open()

> **open**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `withApp?`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### withApp?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### openPath()

> **openPath**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `withApp?`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### withApp?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `system` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
