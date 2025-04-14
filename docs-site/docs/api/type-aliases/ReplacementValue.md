# Type Alias: ReplacementValue

> **ReplacementValue** = [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`Image`](../classes/Image.md) \| [`Macro`](../classes/Macro.md)

A value that can replace typed text: a string, an [Image](../classes/Image.md), or a [Macro](../classes/Macro.md).

```ts
keyboard.onText("btw", "by the way"); // string
keyboard.onText("logo", myImage);     // image
keyboard.onText("greet", myMacro);    // macro
```
