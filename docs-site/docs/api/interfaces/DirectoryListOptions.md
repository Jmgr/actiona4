# Interface: DirectoryListOptions

Defined in: [index.d.ts:3261](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3261)

Options for `Directory.listEntries()`.

```ts
const entries = await Directory.listEntries("/tmp", {
sort: false,
absolutePath: false,
fetchSize: true,
});
```

## Properties

### absolutePath?

> `optional` **absolutePath**: `boolean`

Defined in: [index.d.ts:3271](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3271)

Should each entry's absolute path be computed?

#### Default Value

`true`

***

### fetchSize?

> `optional` **fetchSize**: `boolean`

Defined in: [index.d.ts:3276](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3276)

Should each entry's size be fetched?

#### Default Value

`true`

***

### sort?

> `optional` **sort**: `boolean`

Defined in: [index.d.ts:3266](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3266)

Should the entries be sorted?

#### Default Value

`true`
