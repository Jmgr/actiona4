# Interface: ColorPickerOptions


Color picker dialog options.

```ts
const color = await ui.colorPicker({
  title: "Choose a color",
  value: new Color(255, 0, 0),
});
```

## Properties

### title?

> `optional` **title?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title displayed in the dialog title bar.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### value?

> `optional` **value?**: [`ColorLike`](../type-aliases/ColorLike.md)

Initial color shown in the picker.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
