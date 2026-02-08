# Interface: Os

OS-level information.

```ts
console.log(system.os.name, system.os.version, system.os.kernelVersion);

const users = await system.os.listUsers();
const groups = await system.os.listGroups();
console.log(users.length, groups.length);
```

## Properties

### bootTime

> `readonly` **bootTime**: [`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

Boot time

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

### kernelVersion?

> `readonly` `optional` **kernelVersion**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Kernel version

***

### longVersion?

> `readonly` `optional` **longVersion**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Long version

***

### name?

> `readonly` `optional` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Name

***

### openFilesLimit?

> `readonly` `optional` **openFilesLimit**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Open files limit

***

### uptime

> `readonly` **uptime**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Uptime in seconds

***

### version?

> `readonly` `optional` **version**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Version

## Methods

### listGroups()

> **listGroups**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Group`](Group.md)[]\>

Groups

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Group`](Group.md)[]\>

***

### listUsers()

> **listUsers**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`User`](User.md)[]\>

Users

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`User`](User.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
