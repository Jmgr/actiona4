# Interface: DirectoryEntry

An entry returned by `Directory.listEntries()`, representing a file, directory,
or symlink within a directory.

```ts
const entries = await Directory.listEntries("/home/user");
for (const entry of entries) {
    println(entry.fileName, entry.isFile, entry.size);
}
```

## Properties

### fileName

> `readonly` **fileName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The file name (last component of the path).

***

### isDirectory

> `readonly` **isDirectory**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether this entry is a directory.

***

### isFile

> `readonly` **isFile**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether this entry is a regular file.

***

### isSymlink

> `readonly` **isSymlink**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether this entry is a symbolic link.

***

### path

> `readonly` **path**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The full path to the entry.

***

### size

> `readonly` **size**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The size of the entry in bytes.

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
