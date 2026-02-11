# Interface: Process

A running process.

```ts
const processes = await system.processes.list();
const process = processes[0];
if (process) {
println(process.pid, process.name, process.status);
}
```

## Properties

### accumulatedCpuTime

> `readonly` **accumulatedCpuTime**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Accumulated CPU time in seconds

***

### cmd

> `readonly` **cmd**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Cmd

***

### cpuUsage

> `readonly` **cpuUsage**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

CPU usage

***

### cwd?

> `readonly` `optional` **cwd**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Cwd

***

### diskUsage

> `readonly` **diskUsage**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DiskUsage`](DiskUsage.md)\>

Disk usage

***

### effectiveGroupId?

> `readonly` `optional` **effectiveGroupId**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Effective group ID

#### Platform

only works on Linux

***

### effectiveUserId?

> `readonly` `optional` **effectiveUserId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Effective user ID

#### Platform

only works on Linux

***

### env

> `readonly` **env**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Env

***

### exe?

> `readonly` `optional` **exe**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Exe

***

### exists

> `readonly` **exists**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Exists

***

### groupId?

> `readonly` `optional` **groupId**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Group ID

#### Platform

only works on Linux

***

### memory

> `readonly` **memory**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Memory

***

### name?

> `readonly` `optional` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### openFiles?

> `readonly` `optional` **openFiles**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Open files

***

### openFilesLimit?

> `readonly` `optional` **openFilesLimit**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Open files limit

***

### parent?

> `readonly` `optional` **parent**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Parent

***

### pid

> `readonly` **pid**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Pid

***

### root?

> `readonly` `optional` **root**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Root

***

### runTime

> `readonly` **runTime**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Run time in seconds

***

### sessionId?

> `readonly` `optional` **sessionId**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Session ID

***

### startTime

> `readonly` **startTime**: [`Object`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Object)

Start time

***

### status

> `readonly` **status**: [`ProcessStatus`](../enumerations/ProcessStatus.md)

Status

***

### userId?

> `readonly` `optional` **userId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

User ID

***

### virtualMemory

> `readonly` **virtualMemory**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Virtual memory

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
