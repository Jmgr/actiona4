# Type Alias: KeyLike

> **KeyLike** = [`Key`](../enumerations/Key.md) \| [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

A key as a [Key](../enumerations/Key.md) enum value, a character string, or a numeric key code.

```ts
keyboard.tap(Key.F5); // enum value
keyboard.tap("a");    // character
keyboard.tap(65);     // key code
```
