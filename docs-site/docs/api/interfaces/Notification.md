# Interface: Notification

The global notification singleton for sending desktop notifications.

## Methods

### capabilities()

> **capabilities**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

Server capabilities.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]\>

#### Platform

only works on Linux

***

### show()

> **show**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`NotificationHandle`](NotificationHandle.md)\>

Shows a desktop notification.

#### Parameters

##### options?

[`NotificationOptions`](NotificationOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`NotificationHandle`](NotificationHandle.md)\>
