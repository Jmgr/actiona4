# Interface: NotificationOptions


Options for a notification.

## Properties

### actionIcons?

> `optional` **actionIcons**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

If `true`, action identifiers may be interpreted as icon names.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### actions?

> `optional` **actions**: [`NotificationAction`](NotificationAction.md)[]

Notification actions.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### appName?

> `optional` **appName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Application name, filled by default with executable name.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### attributionText?

> `optional` **attributionText**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Attribution text displayed at the bottom of the notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### autoIcon?

> `optional` **autoIcon**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether to set the icon automatically from executable name.

#### Default Value

`false`

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### body?

> `optional` **body**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Body text of the notification.
Multiple lines possible, may support simple markup.
On Linux, check `notification.capabilities()` for a list.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### category?

> `optional` **category**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Notification category such as `email`, `im`, or `device`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### customHints?

> `optional` **customHints**: [`NotificationCustomHint`](NotificationCustomHint.md)[]

Custom string key/value pairs forwarded as-is.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### customIntHints?

> `optional` **customIntHints**: [`NotificationCustomIntHint`](NotificationCustomIntHint.md)[]

Custom integer key/value pairs forwarded as-is.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### desktopEntry?

> `optional` **desktopEntry**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Desktop entry id (usually app `.desktop` name without extension).

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### group?

> `optional` **group**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Group identifier for organizing notifications.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### header?

> `optional` **header**: [`NotificationHeader`](NotificationHeader.md)

Header for grouping notifications in Action Center.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### heroImage?

> `optional` **heroImage**: [`Image`](../classes/Image.md)

Hero image displayed prominently at the top of the notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### icon?

> `optional` **icon**: [`Image`](../classes/Image.md)

Icon image to display with the notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### iconCropCircle?

> `optional` **iconCropCircle**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether to crop the icon into a circle.

#### Default Value

`false`

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### iconName?

> `optional` **iconName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Icon name/path assigned to the notification icon field.
Usually available in /usr/share/icons.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### inputs?

> `optional` **inputs**: [`NotificationInput`](NotificationInput.md)[]

Input fields displayed in the notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### launch?

> `optional` **launch**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Launch string passed to the app when the notification is clicked.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### point?

> `optional` **point**: [`Point`](../classes/Point.md)

Target screen position for the notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### remoteId?

> `optional` **remoteId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Remote ID for cross-device notification correlation.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### resident?

> `optional` **resident**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

If `true`, keep notification resident until explicitly dismissed.
Also automatically sets the timeout to never expire unless an explicit
timeout is provided.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### scenario?

> `optional` **scenario**: [`NotificationScenario`](../enumerations/NotificationScenario.md)

Toast scenario that adjusts notification behavior.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### selections?

> `optional` **selections**: [`NotificationSelection`](NotificationSelection.md)[]

Selection options for dropdown inputs.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### silent?

> `optional` **silent**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether to suppress all notification sound.

#### Default Value

`false`

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### sound?

> `optional` **sound**: [`NotificationSound`](../enumerations/NotificationSound.md)

Sound to play with the notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### soundFile?

> `optional` **soundFile**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Absolute path to a sound file to play for this notification.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### soundLooping?

> `optional` **soundLooping**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether to loop the notification sound.

#### Default Value

`false`

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### soundName?

> `optional` **soundName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Themeable freedesktop sound name, e.g. `message-new-instant`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### suppressSound?

> `optional` **suppressSound**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

If `true`, suppress notification sounds.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### tag?

> `optional` **tag**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Tag for identifying and replacing notifications.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>

***

### timeout?

> `optional` **timeout**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Timeout before the notification is automatically dismissed.
Note that most servers don't respect this setting.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### title?

> `optional` **title**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Title of the notification (summary line).

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### transient?

> `optional` **transient**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

If `true`, request non-persistent behavior from the server.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### urgency?

> `optional` **urgency**: [`NotificationUrgency`](../enumerations/NotificationUrgency.md)

Urgency level.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Linux"><span class="platform-badge__label">Linux-only</span></span>
</div>

***

### useButtonStyle?

> `optional` **useButtonStyle**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether to enable button styling on actions.

#### Default Value

`false`

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Only works on Windows"><span class="platform-badge__label">Windows-only</span></span>
</div>
