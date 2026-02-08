# Interface: DirectoryEntry

Defined in: [index.d.ts:3207](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3207)

An entry returned by `Directory.listEntries()`, representing a file, directory,
or symlink within a directory.

```ts
const entries = await Directory.listEntries("/home/user");
for (const entry of entries) {
console.log(entry.fileName, entry.isFile, entry.size);
}
```

## Properties

### fileName

> `readonly` **fileName**: `string`

Defined in: [index.d.ts:3215](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3215)

The file name (last component of the path).

***

### isDirectory

> `readonly` **isDirectory**: `boolean`

Defined in: [index.d.ts:3223](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3223)

Whether this entry is a directory.

***

### isFile

> `readonly` **isFile**: `boolean`

Defined in: [index.d.ts:3219](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3219)

Whether this entry is a regular file.

***

### isSymlink

> `readonly` **isSymlink**: `boolean`

Defined in: [index.d.ts:3227](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3227)

Whether this entry is a symbolic link.

***

### path

> `readonly` **path**: `string`

Defined in: [index.d.ts:3211](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3211)

The full path to the entry.

***

### size

> `readonly` **size**: `number`

Defined in: [index.d.ts:3231](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3231)

The size of the entry in bytes.
