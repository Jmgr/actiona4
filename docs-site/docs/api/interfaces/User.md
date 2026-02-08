# Interface: User

Defined in: [index.d.ts:6178](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6178)

A system user.

```ts
const users = await system.os.listUsers();
const user = users[0];
if (user) {
console.log(user.id, user.name, user.groupName);
}
```

## Properties

### groupId?

> `readonly` `optional` **groupId**: `number`

Defined in: [index.d.ts:6191](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6191)

Group ID

#### Platform

does not work on Windows

***

### groupName?

> `readonly` `optional` **groupName**: `string`

Defined in: [index.d.ts:6196](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6196)

Group name

#### Platform

does not work on Windows

***

### groupNames

> `readonly` **groupNames**: readonly `string`[]

Defined in: [index.d.ts:6204](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6204)

Group names

***

### groups

> `readonly` **groups**: readonly `number`[]

Defined in: [index.d.ts:6200](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6200)

Groups

***

### id

> `readonly` **id**: `string`

Defined in: [index.d.ts:6186](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6186)

ID

***

### name

> `readonly` **name**: `string`

Defined in: [index.d.ts:6182](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6182)

Name

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6205](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6205)

#### Returns

`string`
