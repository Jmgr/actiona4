# Class: AbortController

Controls cancellation of asynchronous operations.

Create an `AbortController`, pass its `signal` to a cancellable operation,
and call `abort()` to cancel it.

```ts
const controller = new AbortController();

// Start a long-running operation
const task = source.findImage(template, { signal: controller.signal });

// Cancel after 5 seconds
await sleep(5000);
controller.abort();
```

## Constructors

### Constructor

> **new AbortController**(): `AbortController`

#### Returns

`AbortController`

## Properties

### signal

> `readonly` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

## Methods

### abort()

> **abort**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Signals cancellation to all operations using this controller's signal.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)
