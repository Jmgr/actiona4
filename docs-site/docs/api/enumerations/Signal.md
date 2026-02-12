# Enumeration: Signal

Unix signal.

```ts
await process.sendSignal(1234, Signal.Term);
```

## Platform

only works on Linux

## Enumeration Members

### Cont

> **Cont**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGCONT` - continue a stopped process.

***

### Hup

> **Hup**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGHUP` - hang up; often used to request config reload.

***

### Int

> **Int**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGINT` - interrupt (like Ctrl-C).

***

### Kill

> **Kill**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGKILL` - force kill immediately.

***

### Quit

> **Quit**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGQUIT` - quit; similar to `SIGINT`, often with core dump.

***

### Stop

> **Stop**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGSTOP` - stop/suspend execution immediately.

***

### Term

> **Term**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTERM` - polite termination request.

***

### Tstp

> **Tstp**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTSTP` - terminal stop (like Ctrl-Z).

***

### Ttin

> **Ttin**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTTIN` - background process attempted terminal input.

***

### Ttou

> **Ttou**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTTOU` - background process attempted terminal output.

***

### Usr1

> **Usr1**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGUSR1` - user-defined signal 1.

***

### Usr2

> **Usr2**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGUSR2` - user-defined signal 2.

***

### Winch

> **Winch**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGWINCH` - terminal window size changed.
