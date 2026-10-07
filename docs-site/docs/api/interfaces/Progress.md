# Interface: Progress

An open progress dialog, returned by `dialogs.progress()`.

Updates are cheap: only the latest value and text are shown, so they can be set in a tight
loop.

```ts
const progress = await dialogs.progress("Working…", { cancellable: true });
progress.value = 0.5;
progress.text = "Halfway there";
progress.value = undefined; // show a busy bar
await progress.close();
```

## Properties

### value

> **value**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

Progress between 0 and 1, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) (or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null)) for a busy bar

***

### text

> **text**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Text shown above the progress bar

***

### cancelled

> `readonly` **cancelled**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the user cancelled or closed the dialog.

## Methods

### waitForCancel()

> <span class="async-badge">async</span> **waitForCancel**(`options?`: [`WaitForCancelOptions`](WaitForCancelOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until the user cancels or closes the dialog, or `close()` is called.

```ts
const progress = await dialogs.progress("Waiting…", { cancellable: true });
await progress.waitForCancel();
```

#### Parameters

##### options?

[`WaitForCancelOptions`](WaitForCancelOptions.md)

<div class="options-fields">

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to stop waiting.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### close()

> <span class="async-badge">async</span> **close**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Closes the dialog, and resolves once it is gone. Does nothing if it is already closed.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this progress dialog.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
