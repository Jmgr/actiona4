# Type Alias: NameLike

> **NameLike** = [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`Wildcard`](../classes/Wildcard.md) \| [`RegExp`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/RegExp)

A name matcher: an exact string, a [Wildcard](../classes/Wildcard.md) pattern, or a [RegExp](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/RegExp).

```ts
windows.find("Notepad");              // exact name
windows.find(new Wildcard("Note*"));  // wildcard
windows.find(/^Note/);                // regex
```
