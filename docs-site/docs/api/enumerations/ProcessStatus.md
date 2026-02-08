# Enumeration: ProcessStatus

Defined in: [index.d.ts:1738](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1738)

Process status.

```ts
const processes = await system.processes.list();
const process = processes[0];
if (process && process.status === ProcessStatus.Run) {
console.log("process is running");
}
```

## Enumeration Members

### Dead

> **Dead**: `number`

Defined in: [index.d.ts:1751](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1751)

***

### Idle

> **Idle**: `number`

Defined in: [index.d.ts:1739](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1739)

***

### LockBlocked

> **LockBlocked**: `number`

Defined in: [index.d.ts:1759](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1759)

***

### Parked

> **Parked**: `number`

Defined in: [index.d.ts:1757](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1757)

***

### Run

> **Run**: `number`

Defined in: [index.d.ts:1741](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1741)

***

### Sleep

> **Sleep**: `number`

Defined in: [index.d.ts:1743](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1743)

***

### Stop

> **Stop**: `number`

Defined in: [index.d.ts:1745](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1745)

***

### Suspended

> **Suspended**: `number`

Defined in: [index.d.ts:1763](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1763)

***

### Tracing

> **Tracing**: `number`

Defined in: [index.d.ts:1749](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1749)

***

### UninterruptibleDiskSleep

> **UninterruptibleDiskSleep**: `number`

Defined in: [index.d.ts:1761](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1761)

***

### Unknown

> **Unknown**: `number`

Defined in: [index.d.ts:1765](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1765)

***

### Wakekill

> **Wakekill**: `number`

Defined in: [index.d.ts:1753](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1753)

***

### Waking

> **Waking**: `number`

Defined in: [index.d.ts:1755](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1755)

***

### Zombie

> **Zombie**: `number`

Defined in: [index.d.ts:1747](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1747)
