# Interface: Os

OS-level information.

```ts
println(system.os.name, system.os.version, system.os.kernelVersion);

const users = await system.os.listUsers();
const groups = await system.os.listGroups();
println(users.length, groups.length);
```

## Properties

### name?

> `readonly` `optional` **name?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### kernelVersion?

> `readonly` `optional` **kernelVersion?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Kernel version

***

### version?

> `readonly` `optional` **version?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Version

***

### longVersion?

> `readonly` `optional` **longVersion?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Long version

***

### distributionId

> `readonly` **distributionId**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Distribution ID

***

### distributionIdLike

> `readonly` **distributionIdLike**: readonly [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)[]

Distribution ID like

***

### kernelLongVersion

> `readonly` **kernelLongVersion**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Kernel long version

***

### uptime

> `readonly` **uptime**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Uptime in seconds

***

### bootTime

> `readonly` **bootTime**: [`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

Boot time

***

### openFilesLimit?

> `readonly` `optional` **openFilesLimit?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Open files limit

## Methods

### listUsers()

> <span class="async-badge">async</span> **listUsers**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`User`](User.md)[]\>

Users

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`User`](User.md)[]\>

***

### listGroups()

> <span class="async-badge">async</span> **listGroups**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Group`](Group.md)[]\>

Groups

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Group`](Group.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this OS.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
