# Interface: StandardPaths

Defined in: [index.d.ts:5606](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5606)

Platform-specific standard directory paths.

All properties return the path as a string, or undefined if unavailable.

```ts
console.log(standardPaths.home);       // e.g. "/home/user"
console.log(standardPaths.downloads);   // e.g. "/home/user/Downloads"
console.log(standardPaths.documents);   // e.g. "/home/user/Documents"
```

## Properties

### cache?

> `readonly` `optional` **cache**: `string`

Defined in: [index.d.ts:5642](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5642)

Cache directory

***

### config?

> `readonly` `optional` **config**: `string`

Defined in: [index.d.ts:5646](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5646)

Config directory

***

### desktop?

> `readonly` `optional` **desktop**: `string`

Defined in: [index.d.ts:5618](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5618)

Desktop directory

***

### documents?

> `readonly` `optional` **documents**: `string`

Defined in: [index.d.ts:5622](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5622)

Documents directory

***

### downloads?

> `readonly` `optional` **downloads**: `string`

Defined in: [index.d.ts:5626](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5626)

Downloads directory

***

### home?

> `readonly` `optional` **home**: `string`

Defined in: [index.d.ts:5610](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5610)

Home directory

***

### localConfig?

> `readonly` `optional` **localConfig**: `string`

Defined in: [index.d.ts:5650](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5650)

Local config directory

***

### music?

> `readonly` `optional` **music**: `string`

Defined in: [index.d.ts:5614](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5614)

Music directory

***

### pictures?

> `readonly` `optional` **pictures**: `string`

Defined in: [index.d.ts:5630](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5630)

Pictures directory

***

### public?

> `readonly` `optional` **public**: `string`

Defined in: [index.d.ts:5634](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5634)

Public directory

***

### videos?

> `readonly` `optional` **videos**: `string`

Defined in: [index.d.ts:5638](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5638)

Videos directory

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5654](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5654)

Returns a string representation of all standard paths.

#### Returns

`string`
