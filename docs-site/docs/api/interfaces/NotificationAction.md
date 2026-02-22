# Interface: NotificationAction


A notification action button.

## Properties

### actionType?

> `optional` **actionType**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Action type string (Windows-specific, e.g. for protocol activation).

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### activationType?

> `optional` **activationType**: [`NotificationActivationType`](../enumerations/NotificationActivationType.md)

Activation type for this action.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### buttonStyle?

> `optional` **buttonStyle**: [`NotificationButtonStyle`](../enumerations/NotificationButtonStyle.md)

Visual style of the button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

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

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

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

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>
