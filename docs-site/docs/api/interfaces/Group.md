# Interface: Group

Defined in: [index.d.ts:6219](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6219)

A system group.

```ts
const groups = await system.os.listGroups();
const group = groups[0];
if (group) {
console.log(group.id, group.name);
}
```

## Properties

### id

> `readonly` **id**: `number`

Defined in: [index.d.ts:6227](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6227)

ID

***

### name

> `readonly` **name**: `string`

Defined in: [index.d.ts:6223](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6223)

Name

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6228](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6228)

#### Returns

`string`
