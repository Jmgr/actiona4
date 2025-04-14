# Enumeration: ProcessStatus

Process status.

```ts
const processes = await system.processes.list();
const process = processes[0];
if (process && process.status === ProcessStatus.Run) {
  println("process is running");
}
```

## Enumeration Members

### Idle

> **Idle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Idle`

***

### Run

> **Run**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Run`

***

### Sleep

> **Sleep**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Sleep`

***

### Stop

> **Stop**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Stop`

***

### Zombie

> **Zombie**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Zombie`

***

### Tracing

> **Tracing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Tracing`

***

### Dead

> **Dead**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Dead`

***

### Wakekill

> **Wakekill**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Wakekill`

***

### Waking

> **Waking**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Waking`

***

### Parked

> **Parked**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Parked`

***

### LockBlocked

> **LockBlocked**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.LockBlocked`

***

### UninterruptibleDiskSleep

> **UninterruptibleDiskSleep**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.UninterruptibleDiskSleep`

***

### Suspended

> **Suspended**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Suspended`

***

### Unknown

> **Unknown**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Unknown`
