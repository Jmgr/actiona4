# Interface: NotificationHandle

A handle for a shown desktop notification.

## Methods

### close()

> **close**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Programmatically closes the notification.

```ts
const handle = await notification.show({ title: "Hello", resident: true });
await sleep("5s");
await handle.close();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### update()

> **update**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Updates the notification with new options.

```ts
const handle = await notification.show({ title: "Initial" });
await handle.update({ title: "Updated", body: "New body" });
```

#### Parameters

##### options?

[`NotificationOptions`](NotificationOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Platform

only works on Linux

***

### waitForAction()

> **waitForAction**(`options?`): [`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null)\>

Waits for an action to be invoked on this notification, or for the notification to close.
Returns the action identifier, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if the notification was closed without an action.

```ts
const handle = await notification.show({ title: "Update available", actions: [{ identifier: "update", label: "Update now" }] });
const action = await handle.waitForAction();
if (action === "update") { await runUpdate(); }
```

#### Parameters

##### options?

[`WaitForActionOptions`](WaitForActionOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null)\>

***

### waitUntilClosed()

> **waitUntilClosed**(`options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until this notification is closed or the optional abort signal is triggered.

```ts
const handle = await notification.show({ title: "Waiting..." });
await handle.waitUntilClosed();
```

#### Parameters

##### options?

[`WaitForActionOptions`](WaitForActionOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
