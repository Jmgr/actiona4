# Interface: User

A system user.

```ts
const users = await system.os.listUsers();
const user = users[0];
if (user) {
  println(user.id, user.name, user.groupName);
}
```

## Properties

### groupId?

> `readonly` `optional` **groupId**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Group ID

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Wayland" aria-label="Supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### groupName?

> `readonly` `optional` **groupName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Group name

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Windows" aria-label="Not supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Wayland" aria-label="Supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### groupNames

> `readonly` **groupNames**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Group names

***

### groups

> `readonly` **groups**: readonly [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)[]

Groups

***

### id

> `readonly` **id**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

ID

***

### name

> `readonly` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
