# Class: AbortController

Defined in: [index.d.ts:4604](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4604)

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

Defined in: [index.d.ts:4606](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4606)

#### Returns

`AbortController`

## Properties

### signal

> `readonly` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

Defined in: [index.d.ts:4605](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4605)

## Methods

### abort()

> **abort**(): `void`

Defined in: [index.d.ts:4610](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4610)

Signals cancellation to all operations using this controller's signal.

#### Returns

`void`
