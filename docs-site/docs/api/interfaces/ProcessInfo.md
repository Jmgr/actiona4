# Interface: ProcessInfo

A process information entry.

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

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### effectiveUserId?

> `readonly` `optional` **effectiveUserId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Effective user ID

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

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

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

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

### kill()

> <span class="async-badge">async</span> **kill**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Kill the process immediately (SIGKILL on Unix, TerminateProcess on Windows).

```ts
// Force-stop a specific PID if it is still running.
const targetPid = 12345;
const proc = (await system.processes.find({ pid: targetPid }))[0];
if (proc) await proc.kill();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### sendSignal()

> <span class="async-badge">async</span> **sendSignal**(`signal`: [`Signal`](../enumerations/Signal.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Send a signal to the process.

```ts
const targetPid = 12345;
const proc = (await system.processes.find({ pid: targetPid }))[0];
if (proc) await proc.sendSignal(Signal.Term);
```

#### Parameters

##### signal

[`Signal`](../enumerations/Signal.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### terminate()

> <span class="async-badge">async</span> **terminate**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Gracefully terminate the process (SIGTERM on Unix, WM_CLOSE on Windows).

```ts
// Ask a specific PID to shut down cleanly.
const targetPid = 12345;
const proc = (await system.processes.find({ pid: targetPid }))[0];
if (proc) await proc.terminate();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
