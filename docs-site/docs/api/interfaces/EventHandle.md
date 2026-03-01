# Interface: EventHandle

A handle to a registered event listener. Call `.cancel()` to unregister it.

```ts
const handle = keyboard.onText("btw", "by the way");
// ... later:
handle.cancel();
```

## Methods

### cancel()

> **cancel**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Unregisters this event listener.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
