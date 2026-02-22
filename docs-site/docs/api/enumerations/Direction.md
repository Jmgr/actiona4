# Enumeration: Direction

**`Expand`**

Direction for key press/release actions.

```ts
// Press and hold a key
await keyboard.key(Key.Shift, Direction.Press);
// Release it
await keyboard.key(Key.Shift, Direction.Release);

// Press and release in one action
await keyboard.key(Key.Return, Direction.Click);
```

## Enumeration Members

### Click

> **Click**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Press

> **Press**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### Release

> **Release**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)
