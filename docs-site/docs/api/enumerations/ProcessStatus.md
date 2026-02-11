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

***

### Idle

> **Idle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### LockBlocked

> **LockBlocked**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Parked

> **Parked**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Run

> **Run**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Sleep

> **Sleep**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Stop

> **Stop**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Suspended

> **Suspended**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Tracing

> **Tracing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### UninterruptibleDiskSleep

> **UninterruptibleDiskSleep**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Unknown

> **Unknown**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Wakekill

> **Wakekill**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Waking

> **Waking**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Zombie

> **Zombie**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)
