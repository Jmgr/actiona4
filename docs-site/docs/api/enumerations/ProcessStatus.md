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

### Dead

> **Dead**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Dead`

***

### Idle

> **Idle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Idle`

***

### LockBlocked

> **LockBlocked**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.LockBlocked`

***

### Parked

> **Parked**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Parked`

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

### Suspended

> **Suspended**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Suspended`

***

### Tracing

> **Tracing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Tracing`

***

### UninterruptibleDiskSleep

> **UninterruptibleDiskSleep**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.UninterruptibleDiskSleep`

***

### Unknown

> **Unknown**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Unknown`

***

### Wakekill

> **Wakekill**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Wakekill`

***

### Waking

> **Waking**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Waking`

***

### Zombie

> **Zombie**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ProcessStatus.Zombie`
