# Enumeration: Direction


Direction for key press/release actions.

```ts
// Press and hold a key
keyboard.key(Key.Shift, Direction.Press);
// Release it
keyboard.key(Key.Shift, Direction.Release);

// Press and release in one action
keyboard.key(Key.Return, Direction.Click);
```

## Enumeration Members

### Click

> **Click**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Click`

***

### Press

> **Press**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Press`

***

### Release

> **Release**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Direction.Release`
