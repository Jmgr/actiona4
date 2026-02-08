# Interface: Process

Defined in: [index.d.ts:6269](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6269)

A running process.

```ts
const processes = await system.processes.list();
const process = processes[0];
if (process) {
console.log(process.pid, process.name, process.status);
}
```

## Properties

### accumulatedCpuTime

> `readonly` **accumulatedCpuTime**: `number`

Defined in: [index.d.ts:6329](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6329)

Accumulated CPU time in seconds

***

### cmd

> `readonly` **cmd**: readonly `string`[]

Defined in: [index.d.ts:6277](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6277)

Cmd

***

### cpuUsage

> `readonly` **cpuUsage**: `number`

Defined in: [index.d.ts:6325](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6325)

CPU usage

***

### cwd?

> `readonly` `optional` **cwd**: `string`

Defined in: [index.d.ts:6293](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6293)

Cwd

***

### diskUsage

> `readonly` **diskUsage**: `Readonly`\<[`DiskUsage`](DiskUsage.md)\>

Defined in: [index.d.ts:6333](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6333)

Disk usage

***

### effectiveGroupId?

> `readonly` `optional` **effectiveGroupId**: `number`

Defined in: [index.d.ts:6352](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6352)

Effective group ID

#### Platform

only works on Linux

***

### effectiveUserId?

> `readonly` `optional` **effectiveUserId**: `string`

Defined in: [index.d.ts:6342](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6342)

Effective user ID

#### Platform

only works on Linux

***

### env

> `readonly` **env**: readonly `string`[]

Defined in: [index.d.ts:6289](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6289)

Env

***

### exe?

> `readonly` `optional` **exe**: `string`

Defined in: [index.d.ts:6281](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6281)

Exe

***

### exists

> `readonly` **exists**: `boolean`

Defined in: [index.d.ts:6360](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6360)

Exists

***

### groupId?

> `readonly` `optional` **groupId**: `number`

Defined in: [index.d.ts:6347](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6347)

Group ID

#### Platform

only works on Linux

***

### memory

> `readonly` **memory**: `number`

Defined in: [index.d.ts:6301](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6301)

Memory

***

### name?

> `readonly` `optional` **name**: `string`

Defined in: [index.d.ts:6273](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6273)

Name

***

### openFiles?

> `readonly` `optional` **openFiles**: `number`

Defined in: [index.d.ts:6364](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6364)

Open files

***

### openFilesLimit?

> `readonly` `optional` **openFilesLimit**: `number`

Defined in: [index.d.ts:6368](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6368)

Open files limit

***

### parent?

> `readonly` `optional` **parent**: `number`

Defined in: [index.d.ts:6309](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6309)

Parent

***

### pid

> `readonly` **pid**: `number`

Defined in: [index.d.ts:6285](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6285)

Pid

***

### root?

> `readonly` `optional` **root**: `string`

Defined in: [index.d.ts:6297](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6297)

Root

***

### runTime

> `readonly` **runTime**: `number`

Defined in: [index.d.ts:6321](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6321)

Run time in seconds

***

### sessionId?

> `readonly` `optional` **sessionId**: `number`

Defined in: [index.d.ts:6356](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6356)

Session ID

***

### startTime

> `readonly` **startTime**: `Object`

Defined in: [index.d.ts:6317](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6317)

Start time

***

### status

> `readonly` **status**: [`ProcessStatus`](../enumerations/ProcessStatus.md)

Defined in: [index.d.ts:6313](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6313)

Status

***

### userId?

> `readonly` `optional` **userId**: `string`

Defined in: [index.d.ts:6337](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6337)

User ID

***

### virtualMemory

> `readonly` **virtualMemory**: `number`

Defined in: [index.d.ts:6305](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6305)

Virtual memory

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6369](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6369)

#### Returns

`string`
