# Class: Directory

Provides static methods for creating, removing, and listing directories.

```ts
// Create a directory (recursively by default)
await Directory.create("/tmp/my/nested/dir");

// List entries in a directory
const entries = await Directory.listEntries("/tmp/my/nested/dir");
for (const entry of entries) {
    println(entry.fileName, entry.isFile ? "file" : "dir");
}

// Remove a directory tree
await Directory.remove("/tmp/my");
```

## Methods

### create()

> `static` **create**(`path`, `options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Creates a directory at the given path. By default, creates parent directories
recursively.

```ts
await Directory.create("/tmp/a/b/c");

// Non-recursive: fails if parent doesn't exist
await Directory.create("/tmp/a/b/c", { recursive: false });
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`DirectoryOptions`](../interfaces/DirectoryOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### listEntries()

> `static` **listEntries**(`path`, `options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`DirectoryEntry`](../interfaces/DirectoryEntry.md)[]\>

Lists all entries in a directory, returning an array of `DirectoryEntry`.

```ts
// List with defaults (sorted, absolute paths, sizes fetched)
const entries = await Directory.listEntries("/home/user/docs");

// Skip size fetching for faster listing
const entries = await Directory.listEntries("/home/user/docs", {
    fetchSize: false,
});
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`DirectoryListOptions`](../interfaces/DirectoryListOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`DirectoryEntry`](../interfaces/DirectoryEntry.md)[]\>

***

### remove()

> `static` **remove**(`path`, `options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Removes a directory. By default, removes all contents recursively.

```ts
await Directory.remove("/tmp/my/dir");

// Non-recursive: fails if the directory is not empty
await Directory.remove("/tmp/my/dir", { recursive: false });
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`DirectoryOptions`](../interfaces/DirectoryOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
