# Interface: FileFilter

A file type filter for file dialogs.

```ts
const filter = { name: "Images", extensions: ["png", "jpg"] };
```

## Properties

### name

> **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Display name of the filter.

***

### extensions

> **extensions**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

File extensions matched by this filter (without leading dot).
