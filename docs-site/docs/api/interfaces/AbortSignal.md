# Interface: AbortSignal

Defined in: [index.d.ts:4584](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4584)

A signal that can be used to abort asynchronous operations.

Obtained from an `AbortController` via the `signal` property. Pass it
to cancellable operations (e.g., `findImage`) in their options.

```ts
const controller = new AbortController();
const task = source.findImage(template, { signal: controller.signal });
// Cancel from elsewhere:
controller.abort();
```
