# Interface: Os

Defined in: [index.d.ts:6115](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6115)

OS-level information.

```ts
console.log(system.os.name, system.os.version, system.os.kernelVersion);

const users = await system.os.listUsers();
const groups = await system.os.listGroups();
console.log(users.length, groups.length);
```

## Properties

### bootTime

> `readonly` **bootTime**: `Date`

Defined in: [index.d.ts:6151](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6151)

Boot time

***

### distributionId

> `readonly` **distributionId**: `string`

Defined in: [index.d.ts:6135](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6135)

Distribution ID

***

### distributionIdLike

> `readonly` **distributionIdLike**: readonly `string`[]

Defined in: [index.d.ts:6139](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6139)

Distribution ID like

***

### kernelLongVersion

> `readonly` **kernelLongVersion**: `string`

Defined in: [index.d.ts:6143](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6143)

Kernel long version

***

### kernelVersion?

> `readonly` `optional` **kernelVersion**: `string`

Defined in: [index.d.ts:6123](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6123)

Kernel version

***

### longVersion?

> `readonly` `optional` **longVersion**: `string`

Defined in: [index.d.ts:6131](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6131)

Long version

***

### name?

> `readonly` `optional` **name**: `string`

Defined in: [index.d.ts:6119](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6119)

Name

***

### openFilesLimit?

> `readonly` `optional` **openFilesLimit**: `number`

Defined in: [index.d.ts:6155](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6155)

Open files limit

***

### uptime

> `readonly` **uptime**: `number`

Defined in: [index.d.ts:6147](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6147)

Uptime in seconds

***

### version?

> `readonly` `optional` **version**: `string`

Defined in: [index.d.ts:6127](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6127)

Version

## Methods

### listGroups()

> **listGroups**(): `Promise`\<readonly [`Group`](Group.md)[]\>

Defined in: [index.d.ts:6163](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6163)

Groups

#### Returns

`Promise`\<readonly [`Group`](Group.md)[]\>

***

### listUsers()

> **listUsers**(): `Promise`\<readonly [`User`](User.md)[]\>

Defined in: [index.d.ts:6159](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6159)

Users

#### Returns

`Promise`\<readonly [`User`](User.md)[]\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6164](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6164)

#### Returns

`string`
