# Class: Directory

Defined in: [index.d.ts:3296](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3296)

Provides static methods for creating, removing, and listing directories.

```ts
// Create a directory (recursively by default)
await Directory.create("/tmp/my/nested/dir");

// List entries in a directory
const entries = await Directory.listEntries("/tmp/my/nested/dir");
for (const entry of entries) {
console.log(entry.fileName, entry.isFile ? "file" : "dir");
}

// Remove a directory tree
await Directory.remove("/tmp/my");
```

## Methods

### create()

> `static` **create**(`path`, `options?`): `Promise`\<`void`\>

Defined in: [index.d.ts:3309](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3309)

Creates a directory at the given path. By default, creates parent directories
recursively.

```ts
await Directory.create("/tmp/a/b/c");

// Non-recursive: fails if parent doesn't exist
await Directory.create("/tmp/a/b/c", { recursive: false });
```

#### Parameters

##### path

`string`

##### options?

[`DirectoryOptions`](../interfaces/DirectoryOptions.md)

#### Returns

`Promise`\<`void`\>

***

### listEntries()

> `static` **listEntries**(`path`, `options?`): `Promise`\<readonly [`DirectoryEntry`](../interfaces/DirectoryEntry.md)[]\>

Defined in: [index.d.ts:3334](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3334)

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

`string`

##### options?

[`DirectoryListOptions`](../interfaces/DirectoryListOptions.md)

#### Returns

`Promise`\<readonly [`DirectoryEntry`](../interfaces/DirectoryEntry.md)[]\>

***

### remove()

> `static` **remove**(`path`, `options?`): `Promise`\<`void`\>

Defined in: [index.d.ts:3320](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3320)

Removes a directory. By default, removes all contents recursively.

```ts
await Directory.remove("/tmp/my/dir");

// Non-recursive: fails if the directory is not empty
await Directory.remove("/tmp/my/dir", { recursive: false });
```

#### Parameters

##### path

`string`

##### options?

[`DirectoryOptions`](../interfaces/DirectoryOptions.md)

#### Returns

`Promise`\<`void`\>
