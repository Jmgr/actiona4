# Enumeration: Signal

Unix signal.

```ts
await process.sendSignal(1234, Signal.Term);
```

## Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux-only</span></span>
</div>

## Enumeration Members

### Cont

> **Cont**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGCONT` - continue a stopped process.
`Signal.Cont`

***

### Hup

> **Hup**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGHUP` - hang up; often used to request config reload.
`Signal.Hup`

***

### Int

> **Int**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGINT` - interrupt (like Ctrl-C).
`Signal.Int`

***

### Kill

> **Kill**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGKILL` - force kill immediately.
`Signal.Kill`

***

### Quit

> **Quit**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGQUIT` - quit; similar to `SIGINT`, often with core dump.
`Signal.Quit`

***

### Stop

> **Stop**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGSTOP` - stop/suspend execution immediately.
`Signal.Stop`

***

### Term

> **Term**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTERM` - polite termination request.
`Signal.Term`

***

### Tstp

> **Tstp**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTSTP` - terminal stop (like Ctrl-Z).
`Signal.Tstp`

***

### Ttin

> **Ttin**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTTIN` - background process attempted terminal input.
`Signal.Ttin`

***

### Ttou

> **Ttou**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGTTOU` - background process attempted terminal output.
`Signal.Ttou`

***

### Usr1

> **Usr1**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGUSR1` - user-defined signal 1.
`Signal.Usr1`

***

### Usr2

> **Usr2**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGUSR2` - user-defined signal 2.
`Signal.Usr2`

***

### Winch

> **Winch**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`SIGWINCH` - terminal window size changed.
`Signal.Winch`
