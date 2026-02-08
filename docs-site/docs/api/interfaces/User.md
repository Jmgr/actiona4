# Interface: User

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

> `readonly` `optional` **groupId**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Group ID

#### Platform

does not work on Windows

***

### groupName?

> `readonly` `optional` **groupName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Group name

#### Platform

does not work on Windows

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
