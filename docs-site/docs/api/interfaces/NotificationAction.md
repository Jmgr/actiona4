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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Linux" aria-label="Not supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### activationType?

> `optional` **activationType**: [`NotificationActivationType`](../enumerations/NotificationActivationType.md)

Activation type for this action.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Linux" aria-label="Not supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### buttonStyle?

> `optional` **buttonStyle**: [`NotificationButtonStyle`](../enumerations/NotificationButtonStyle.md)

Visual style of the button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Linux" aria-label="Not supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### identifier?

> `optional` **identifier**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Action identifier (used as arguments on Windows).

#### Default Value

`""`

***

### inputId?

> `optional` **inputId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

ID of the input element this action is associated with.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Linux" aria-label="Not supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Linux</span></span>
</div>

***

### label?

> `optional` **label**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Action label visible to the user.

#### Default Value

`""`

***

### placement?

> `optional` **placement**: [`ContextMenu`](../enumerations/NotificationActionPlacement.md#contextmenu)

Placement of this action button.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Linux" aria-label="Not supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Linux</span></span>
</div>
