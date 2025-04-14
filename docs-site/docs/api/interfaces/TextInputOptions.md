# Interface: TextInputOptions


Text input dialog options.

```ts
const name = await ui.textInput("Enter your name:", {
  title: "Name",
  mode: TextInputMode.SingleLine,
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

> `optional` **value?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Initial value shown in the text field.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### mode?

> `optional` **mode?**: [`TextInputMode`](../enumerations/TextInputMode.md)

Input mode controlling the dialog style.

#### Default Value

`TextInputMode.SingleLine`
