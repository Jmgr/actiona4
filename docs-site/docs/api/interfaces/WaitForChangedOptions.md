# Interface: WaitForChangedOptions


Options for waiting until clipboard content changes.

```ts
// Wait for any clipboard change
await clipboard.waitForChanged();

// Wait on Linux selection clipboard with a custom polling interval
await clipboard.waitForChanged({ mode: ClipboardMode.Selection, interval: 0.05 });

// Wait up to 1 second for a clipboard change
await Concurrency.race([
  clipboard.waitForChanged(),
  sleep("1s"),
]);
```

## Properties

### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Polling interval in seconds.

#### Default Value

`0.2`

***

### mode?

> `optional` **mode**: [`ClipboardMode`](../enumerations/ClipboardMode.md)

Clipboard source to watch.

#### Default Value

`ClipboardMode.Clipboard`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
