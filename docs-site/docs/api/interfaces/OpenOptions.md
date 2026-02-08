# Interface: OpenOptions

Defined in: [index.d.ts:3470](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3470)

Options for `File.open()`.

```ts
// Read-only (default)
const file = await File.open("data.txt");

// Create a new file for writing
const file = await File.open("out.txt", {
write: true,
createNew: true,
});

// Append to an existing file
const file = await File.open("log.txt", {
write: true,
append: true,
});
```

## Properties

### append?

> `optional` **append**: `boolean`

Defined in: [index.d.ts:3486](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3486)

Writing: open the file in append mode.
Note that setting this to `true` implies setting `write` to `true`.

#### Default Value

`false`

***

### create?

> `optional` **create**: `boolean`

Defined in: [index.d.ts:3498](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3498)

Writing: create a new file if it doesn't exist.
Note that this only works if `write` or `append` are set to `true`.

#### Default Value

`false`

***

### createNew?

> `optional` **createNew**: `boolean`

Defined in: [index.d.ts:3505](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3505)

Writing: always create a new file, even if one already exists.
Note that this only works if `write` or `append` are set to `true`.
Note that `create` and `truncate` are ignored if this is set to `true`.

#### Default Value

`false`

***

### read?

> `optional` **read**: `boolean`

Defined in: [index.d.ts:3475](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3475)

Should the file be opened with read access?

#### Default Value

`true`

***

### truncate?

> `optional` **truncate**: `boolean`

Defined in: [index.d.ts:3492](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3492)

Writing: truncate (remove all contents of) the file.
Note that this only works if `write` is `true`.

#### Default Value

`false`

***

### write?

> `optional` **write**: `boolean`

Defined in: [index.d.ts:3480](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3480)

Should the file be opened with write access?

#### Default Value

`false`
