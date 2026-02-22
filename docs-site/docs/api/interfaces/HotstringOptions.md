# Interface: HotstringOptions


Hotstring options

## Properties

### eraseKey?

> `optional` **eraseKey**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Erase the key first before replacing it with the replacement content.

#### Default Value

`true`

***

### saveRestoreClipboard?

> `optional` **saveRestoreClipboard**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Try to save and restore the clipboard's contents.

#### Default Value

`true`

***

### useClipboardForText?

> `optional` **useClipboardForText**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

When replacing with text, save it to the clipboard then simulate Ctrl+V to paste.
Replacing with an image always uses the clipboard.

#### Default Value

`false`
