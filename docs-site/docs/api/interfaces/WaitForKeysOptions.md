# Interface: WaitForKeysOptions

Defined in: [index.d.ts:4708](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4708)

Options for waiting for key combinations.

```ts
// Wait for exactly Ctrl+S and no other keys
await keyboard.waitForKeys([Key.Control, "s"], { exclusive: true });
```

## Properties

### exclusive?

> `optional` **exclusive**: `boolean`

Defined in: [index.d.ts:4713](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4713)

Wait for exactly these keys and no other

#### Default Value

`false`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:4718](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4718)

Abort signal to cancel the wait.

#### Default Value

`undefined`
