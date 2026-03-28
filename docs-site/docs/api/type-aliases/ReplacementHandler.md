# Type Alias: ReplacementHandler

> **ReplacementHandler** = [`ReplacementValue`](ReplacementValue.md) \| (() => [`ReplacementValue`](ReplacementValue.md) \| [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`ReplacementValue`](ReplacementValue.md) \| [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>)

A text replacement: either a [ReplacementValue](ReplacementValue.md) directly, or a (possibly async)
function that returns one.

```ts
keyboard.onText("btw", "by the way");                                 // direct value
keyboard.onText("date", () => new Date().toLocaleDateString());        // function
keyboard.onText("img", async () => await Image.load("logo.png"));      // async function
```
