# Interface: KeysOptions


Options for key-based methods: `onKey`, `onKeys`, and `waitForKeys`.

```ts
// Wait for exactly Ctrl+S and no other keys
await keyboard.waitForKeys([Key.Control, "s"], { exclusive: true });
```

## Properties

### exclusive?

> `optional` **exclusive**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Require exactly these keys and no others to be pressed simultaneously.

#### Default Value

`false`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the operation.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
