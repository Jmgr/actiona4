# Enumeration: Direction

Defined in: [index.d.ts:225](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L225)

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

> **Click**: `number`

Defined in: [index.d.ts:230](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L230)

***

### Press

> **Press**: `number`

Defined in: [index.d.ts:226](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L226)

***

### Release

> **Release**: `number`

Defined in: [index.d.ts:228](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L228)
