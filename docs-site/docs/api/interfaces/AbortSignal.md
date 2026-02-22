# Interface: AbortSignal

A signal that can be used to abort asynchronous operations.

Obtained from an `AbortController` via the `signal` property. Pass it
to cancellable operations (e.g., `findImage`) in their options.

```ts
const controller = new AbortController();
const task = source.findImage(template, { signal: controller.signal });
// Cancel from elsewhere:
controller.abort();
```

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
