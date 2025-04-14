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

### name?

> `readonly` `optional` **name?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### cmd

> `readonly` **cmd**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Cmd

***

### exe?

> `readonly` `optional` **exe?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Exe

***

### pid

> `readonly` **pid**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Pid

***

### env

> `readonly` **env**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Env

***

### cwd?

> `readonly` `optional` **cwd?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Cwd

***

### root?

> `readonly` `optional` **root?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Root

***

### memory

> `readonly` **memory**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Memory

***

### virtualMemory

> `readonly` **virtualMemory**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Virtual memory

***

### parent?

> `readonly` `optional` **parent?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Parent

***

### status

> `readonly` **status**: [`ProcessStatus`](../enumerations/ProcessStatus.md)

Status

***

### startTime

> `readonly` **startTime**: [`Object`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Object)

Start time

***

### runTime

> `readonly` **runTime**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Run time in seconds

***

### cpuUsage

> `readonly` **cpuUsage**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

CPU usage

***

### accumulatedCpuTime

> `readonly` **accumulatedCpuTime**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Accumulated CPU time in seconds

***

### diskUsage

> `readonly` **diskUsage**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DiskUsage`](DiskUsage.md)\>

Disk usage

***

### userId?

> `readonly` `optional` **userId?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

User ID

***

### effectiveUserId?

> `readonly` `optional` **effectiveUserId?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Effective user ID

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### groupId?

> `readonly` `optional` **groupId?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Group ID

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### effectiveGroupId?

> `readonly` `optional` **effectiveGroupId?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Effective group ID

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### sessionId?

> `readonly` `optional` **sessionId?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Session ID

***

### exists

> `readonly` **exists**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Exists

***

### openFiles?

> `readonly` `optional` **openFiles?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Open files

***

### openFilesLimit?

> `readonly` `optional` **openFilesLimit?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Open files limit

## Methods

### kill()

> **kill**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Kill the process immediately (SIGKILL on Unix, TerminateProcess on Windows).

```ts
// Force-stop a specific PID if it is still running.
const targetPid = 12345;
const proc = (await system.processes.find({ pid: targetPid }))[0];
if (proc) proc.kill();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### terminate()

> **terminate**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Gracefully terminate the process (SIGTERM on Unix, WM_CLOSE on Windows).

```ts
// Ask a specific PID to shut down cleanly.
const targetPid = 12345;
const proc = (await system.processes.find({ pid: targetPid }))[0];
if (proc) proc.terminate();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### sendSignal()

> **sendSignal**(`signal`: [`Signal`](../enumerations/Signal.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Send a signal to the process.

```ts
const targetPid = 12345;
const proc = (await system.processes.find({ pid: targetPid }))[0];
if (proc) proc.sendSignal(Signal.Term);
```

#### Parameters

##### signal

[`Signal`](../enumerations/Signal.md)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this process info.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
