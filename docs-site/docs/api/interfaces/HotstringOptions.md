# Interface: HotstringOptions

Defined in: [index.d.ts:7051](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7051)

Hotstring options

## Properties

### eraseKey?

> `optional` **eraseKey**: `boolean`

Defined in: [index.d.ts:7056](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7056)

Erase the key first before replacing it with the replacement content.

#### Default Value

`true`

***

### saveRestoreClipboard?

> `optional` **saveRestoreClipboard**: `boolean`

Defined in: [index.d.ts:7067](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7067)

Try to save and restore the clipboard's contents.

#### Default Value

`true`

***

### useClipboardForText?

> `optional` **useClipboardForText**: `boolean`

Defined in: [index.d.ts:7062](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7062)

When replacing with text, save it to the clipboard then simulate Ctrl+V to paste.
Replacing with an image always uses the clipboard.

#### Default Value

`false`
