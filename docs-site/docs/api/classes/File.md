# Class: File

Defined in: [index.d.ts:3533](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3533)

A file handle for reading and writing. Also provides static utility methods
for common file operations without needing to open a handle.

```ts
// Read a file in one shot (static)
const text = await File.readText("config.json");

// Write a file in one shot (static)
await File.writeText("output.txt", "Hello!");

// Open, read/write, then close
const file = await File.open("data.bin", { read: true, write: true, create: true });
await file.writeBytes(new Uint8Array([1, 2, 3]));
await file.rewind();
const bytes = await file.readBytes();
await file.close();

// File utilities
await File.copy("src.txt", "dst.txt");
await File.rename("old.txt", "new.txt");
const exists = await File.exists("file.txt");
await File.remove("file.txt");
```

## Properties

### path

> `readonly` **path**: `string`

Defined in: [index.d.ts:3537](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3537)

The file path

## Methods

### accessedTime()

> **accessedTime**(): `Promise`\<`Date`\>

Defined in: [index.d.ts:3652](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3652)

Returns the last access time of the file.

#### Returns

`Promise`\<`Date`\>

***

### clone()

> **clone**(): `File`

Defined in: [index.d.ts:3713](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3713)

Returns a clone of this file handle. Both handles share the same underlying file.

#### Returns

`File`

***

### close()

> **close**(): `void`

Defined in: [index.d.ts:3572](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3572)

Closes this file handle.
Please note that the actual file might not be closed until all other handles to it are also closed.
This can happen if you cloned() this File.

#### Returns

`void`

***

### creationTime()

> **creationTime**(): `Promise`\<`Date`\>

Defined in: [index.d.ts:3660](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3660)

Returns the creation time of the file.

#### Returns

`Promise`\<`Date`\>

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:3717](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3717)

Returns `true` if both handles refer to the same file path.

#### Parameters

##### other

`File`

#### Returns

`boolean`

***

### isOpen()

> **isOpen**(): `boolean`

Defined in: [index.d.ts:3566](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3566)

Returns true if the file is open.

#### Returns

`boolean`

***

### mode()

> **mode**(): `Promise`\<`number`\>

Defined in: [index.d.ts:3634](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3634)

Returns the Unix file mode (e.g. `0o644`). Returns `0` on Windows.

#### Returns

`Promise`\<`number`\>

#### Platform

does not work on Windows

***

### modifiedTime()

> **modifiedTime**(): `Promise`\<`Date`\>

Defined in: [index.d.ts:3644](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3644)

Returns the last modification time of the file.

#### Returns

`Promise`\<`Date`\>

***

### position()

> **position**(): `Promise`\<`number`\>

Defined in: [index.d.ts:3669](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3669)

Returns the current read/write position in the file.

#### Returns

`Promise`\<`number`\>

***

### readBytes()

> **readBytes**(`amount?`): `Promise`\<`Uint8Array`\>

Defined in: [index.d.ts:3597](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3597)

Reads bytes from this file handle. If `amount` is given, reads exactly that many bytes;
otherwise reads until EOF.

#### Parameters

##### amount?

`number`

#### Returns

`Promise`\<`Uint8Array`\>

***

### readonly()

> **readonly**(): `Promise`\<`boolean`\>

Defined in: [index.d.ts:3625](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3625)

Returns whether the file is read-only.

#### Returns

`Promise`\<`boolean`\>

***

### readText()

> **readText**(): `Promise`\<`string`\>

Defined in: [index.d.ts:3605](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3605)

Reads the entire file as a UTF-8 string from this file handle.

#### Returns

`Promise`\<`string`\>

***

### rewind()

> **rewind**(): `Promise`\<`void`\>

Defined in: [index.d.ts:3681](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3681)

Rewinds the file position to the beginning.

#### Returns

`Promise`\<`void`\>

***

### setAccessedTime()

> **setAccessedTime**(`date`): `Promise`\<`void`\>

Defined in: [index.d.ts:3656](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3656)

Sets the last access time of the file.

#### Parameters

##### date

`Date`

#### Returns

`Promise`\<`void`\>

***

### setCreationTime()

> **setCreationTime**(`date`): `Promise`\<`void`\>

Defined in: [index.d.ts:3665](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3665)

Sets the creation time of the file. No-op on Linux.

#### Parameters

##### date

`Date`

#### Returns

`Promise`\<`void`\>

#### Platform

does not work on Linux

***

### setMode()

> **setMode**(`mode`): `Promise`\<`void`\>

Defined in: [index.d.ts:3640](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3640)

Sets the file mode.
You should use the octal notation to specify the mode: `await file.setMode(0o445)`.

#### Parameters

##### mode

`number`

#### Returns

`Promise`\<`void`\>

#### Platform

does not work on Windows

***

### setModifiedTime()

> **setModifiedTime**(`date`): `Promise`\<`void`\>

Defined in: [index.d.ts:3648](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3648)

Sets the last modification time of the file.

#### Parameters

##### date

`Date`

#### Returns

`Promise`\<`void`\>

***

### setPosition()

> **setPosition**(`position`): `Promise`\<`void`\>

Defined in: [index.d.ts:3673](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3673)

Seeks to an absolute position in the file.

#### Parameters

##### position

`number`

#### Returns

`Promise`\<`void`\>

***

### setReadonly()

> **setReadonly**(`readonly`): `Promise`\<`void`\>

Defined in: [index.d.ts:3629](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3629)

Sets whether the file is read-only.

#### Parameters

##### readonly

`boolean`

#### Returns

`Promise`\<`void`\>

***

### setRelativePosition()

> **setRelativePosition**(`offset`): `Promise`\<`void`\>

Defined in: [index.d.ts:3677](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3677)

Seeks relative to the current position (can be negative).

#### Parameters

##### offset

`number`

#### Returns

`Promise`\<`void`\>

***

### setSize()

> **setSize**(`size`): `Promise`\<`void`\>

Defined in: [index.d.ts:3621](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3621)

Truncates or extends the file to the given size in bytes.

#### Parameters

##### size

`number`

#### Returns

`Promise`\<`void`\>

***

### size()

> **size**(): `Promise`\<`number`\>

Defined in: [index.d.ts:3617](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3617)

Returns the file size in bytes.

#### Returns

`Promise`\<`number`\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:3721](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3721)

Returns a string representation of the file handle.

#### Returns

`string`

***

### writeBytes()

> **writeBytes**(`bytes`): `Promise`\<`void`\>

Defined in: [index.d.ts:3576](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3576)

Writes bytes to this file handle.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Promise`\<`void`\>

***

### writeText()

> **writeText**(`text`): `Promise`\<`void`\>

Defined in: [index.d.ts:3584](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3584)

Writes text to this file handle.

#### Parameters

##### text

`string`

#### Returns

`Promise`\<`void`\>

***

### copy()

> `static` **copy**(`source`, `destination`): `Promise`\<`void`\>

Defined in: [index.d.ts:3701](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3701)

Copies a file from `source` to `destination`.

#### Parameters

##### source

`string`

##### destination

`string`

#### Returns

`Promise`\<`void`\>

***

### exists()

> `static` **exists**(`path`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:3691](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3691)

Returns `true` if a file exists at the given path.

```ts
if (await File.exists("config.json")) {
const text = await File.readText("config.json");
}
```

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`boolean`\>

***

### move()

> `static` **move**(`source`, `destination`): `Promise`\<`void`\>

Defined in: [index.d.ts:3709](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3709)

Alias for `rename`.

#### Parameters

##### source

`string`

##### destination

`string`

#### Returns

`Promise`\<`void`\>

***

### open()

> `static` **open**(`path`, `options?`): `Promise`\<`File`\>

Defined in: [index.d.ts:3562](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3562)

Opens a file.

Example
```js
// Open a file for reading
let file = await File.open("my_file.txt", {
read: true,
});

// Create a new file for writing.
let file = await File.open("my_file.txt", {
write: true,
createNew: true,
});

// Append to an existing file.
let file = await File.open("my_file.txt", {
write: true,
append: true,
});
```

#### Parameters

##### path

`string`

##### options?

[`OpenOptions`](../interfaces/OpenOptions.md)

#### Returns

`Promise`\<`File`\>

***

### readBytes()

> `static` **readBytes**(`path`, `amount?`): `Promise`\<`Uint8Array`\>

Defined in: [index.d.ts:3601](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3601)

Reads bytes from a file at the given path (static).

#### Parameters

##### path

`string`

##### amount?

`number`

#### Returns

`Promise`\<`Uint8Array`\>

***

### readText()

> `static` **readText**(`path`): `Promise`\<`string`\>

Defined in: [index.d.ts:3613](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3613)

Reads the entire file as a UTF-8 string (static).

```ts
const text = await File.readText("config.json");
```

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`string`\>

***

### remove()

> `static` **remove**(`path`): `Promise`\<`void`\>

Defined in: [index.d.ts:3697](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3697)

Removes a file from the filesystem.

Note that there is no guarantee that the file is immediately deleted (e.g. depending on platform, other open file descriptors may prevent immediate removal).

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`void`\>

***

### rename()

> `static` **rename**(`source`, `destination`): `Promise`\<`void`\>

Defined in: [index.d.ts:3705](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3705)

Renames (moves) a file from `source` to `destination`. Works across filesystems.

#### Parameters

##### source

`string`

##### destination

`string`

#### Returns

`Promise`\<`void`\>

***

### writeBytes()

> `static` **writeBytes**(`path`, `bytes`): `Promise`\<`void`\>

Defined in: [index.d.ts:3580](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3580)

Writes bytes to a file at the given path (static).

#### Parameters

##### path

`string`

##### bytes

`Uint8Array`

#### Returns

`Promise`\<`void`\>

***

### writeText()

> `static` **writeText**(`path`, `text`): `Promise`\<`void`\>

Defined in: [index.d.ts:3592](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3592)

Writes text to a file at the given path (static).

```ts
await File.writeText("hello.txt", "Hello, world!");
```

#### Parameters

##### path

`string`

##### text

`string`

#### Returns

`Promise`\<`void`\>
