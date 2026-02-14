# Interface: NotificationAction

A notification action button.

## Properties

### actionType?

> `optional` **actionType**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Action type string (Windows-specific, e.g. for protocol activation).

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

only works on Windows

***

### activationType?

> `optional` **activationType**: [`NotificationActivationType`](../enumerations/NotificationActivationType.md)

Activation type for this action.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

only works on Windows

***

### buttonStyle?

> `optional` **buttonStyle**: [`NotificationButtonStyle`](../enumerations/NotificationButtonStyle.md)

Visual style of the button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

only works on Windows

***

### identifier?

> `optional` **identifier**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Action identifier (used as arguments on Windows).

***

### inputId?

> `optional` **inputId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

ID of the input element this action is associated with.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

only works on Windows

***

### label?

> `optional` **label**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Action label visible to the user.

***

### placement?

> `optional` **placement**: [`ContextMenu`](../enumerations/NotificationActionPlacement.md#contextmenu)

Placement of this action button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

only works on Windows
