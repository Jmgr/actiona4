# Interface: OnTextOptions


Options for `onText`.

## Properties

### erase?

> `optional` **erase?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Erase the typed text before inserting the replacement.
Set to `false` to trigger an action without replacing the typed text.

#### Default Value

`true`

***

### saveRestoreClipboard?

> `optional` **saveRestoreClipboard?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Save and restore the clipboard contents around a clipboard-based replacement.

#### Default Value

`true`

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to automatically cancel this listener when signalled.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### useClipboardForText?

> `optional` **useClipboardForText?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

When replacing with text, use the clipboard (Ctrl+V) instead of simulated keystrokes.
Replacing with an image always uses the clipboard.

#### Default Value

`false`
