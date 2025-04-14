# Interface: FileDialogOptions


File dialog options.

```ts
const path = await ui.pickFile({
  title: "Open Image",
  filters: [{ name: "Images", extensions: ["png", "jpg"] }],
});
```

## Properties

### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### directory?

> `optional` **directory?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial directory shown in the dialog.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### filters?

> `optional` **filters?**: [`FileFilter`](FileFilter.md)[]

File type filters shown in the dialog.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
