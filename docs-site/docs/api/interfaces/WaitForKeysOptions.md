# Interface: WaitForKeysOptions

Options for waiting for key combinations.

```ts
// Wait for exactly Ctrl+S and no other keys
await keyboard.waitForKeys([Key.Control, "s"], { exclusive: true });
```

## Properties

### exclusive?

> `optional` **exclusive**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Wait for exactly these keys and no other

#### Default Value

`false`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
