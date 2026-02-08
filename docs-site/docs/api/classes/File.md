# Class: File

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

> `readonly` **path**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The file path

## Methods

### accessedTime()

> **accessedTime**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)\>

Returns the last access time of the file.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)\>

***

### clone()

> **clone**(): `File`

Returns a clone of this file handle. Both handles share the same underlying file.

#### Returns

`File`

***

### close()

> **close**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Closes this file handle.
Please note that the actual file might not be closed until all other handles to it are also closed.
This can happen if you cloned() this File.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### creationTime()

> **creationTime**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)\>

Returns the creation time of the file.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)\>

***

### equals()

> **equals**(`other`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns `true` if both handles refer to the same file path.

#### Parameters

##### other

`File`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### isOpen()

> **isOpen**(): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if the file is open.

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### mode()

> **mode**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Returns the Unix file mode (e.g. `0o644`). Returns `0` on Windows.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

#### Platform

does not work on Windows

***

### modifiedTime()

> **modifiedTime**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)\>

Returns the last modification time of the file.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)\>

***

### position()

> **position**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Returns the current read/write position in the file.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

***

### readBytes()

> **readBytes**(`amount?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)\>

Reads bytes from this file handle. If `amount` is given, reads exactly that many bytes;
otherwise reads until EOF.

#### Parameters

##### amount?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)\>

***

### readonly()

> **readonly**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns whether the file is read-only.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

***

### readText()

> **readText**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

Reads the entire file as a UTF-8 string from this file handle.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

***

### rewind()

> **rewind**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Rewinds the file position to the beginning.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setAccessedTime()

> **setAccessedTime**(`date`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the last access time of the file.

#### Parameters

##### date

[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setCreationTime()

> **setCreationTime**(`date`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the creation time of the file. No-op on Linux.

#### Parameters

##### date

[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Platform

does not work on Linux

***

### setMode()

> **setMode**(`mode`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the file mode.
You should use the octal notation to specify the mode: `await file.setMode(0o445)`.

#### Parameters

##### mode

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Platform

does not work on Windows

***

### setModifiedTime()

> **setModifiedTime**(`date`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the last modification time of the file.

#### Parameters

##### date

[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setPosition()

> **setPosition**(`position`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Seeks to an absolute position in the file.

#### Parameters

##### position

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setReadonly()

> **setReadonly**(`readonly`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets whether the file is read-only.

#### Parameters

##### readonly

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setRelativePosition()

> **setRelativePosition**(`offset`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Seeks relative to the current position (can be negative).

#### Parameters

##### offset

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setSize()

> **setSize**(`size`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Truncates or extends the file to the given size in bytes.

#### Parameters

##### size

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### size()

> **size**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Returns the file size in bytes.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the file handle.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### writeBytes()

> **writeBytes**(`bytes`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Writes bytes to this file handle.

#### Parameters

##### bytes

[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### writeText()

> **writeText**(`text`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Writes text to this file handle.

#### Parameters

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### copy()

> `static` **copy**(`source`, `destination`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Copies a file from `source` to `destination`.

#### Parameters

##### source

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### destination

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### exists()

> `static` **exists**(`path`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns `true` if a file exists at the given path.

```ts
if (await File.exists("config.json")) {
const text = await File.readText("config.json");
}
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

***

### move()

> `static` **move**(`source`, `destination`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Alias for `rename`.

#### Parameters

##### source

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### destination

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### open()

> `static` **open**(`path`, `options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`File`\>

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

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`OpenOptions`](../interfaces/OpenOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`File`\>

***

### readBytes()

> `static` **readBytes**(`path`, `amount?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)\>

Reads bytes from a file at the given path (static).

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### amount?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)\>

***

### readText()

> `static` **readText**(`path`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

Reads the entire file as a UTF-8 string (static).

```ts
const text = await File.readText("config.json");
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)\>

***

### remove()

> `static` **remove**(`path`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Removes a file from the filesystem.

Note that there is no guarantee that the file is immediately deleted (e.g. depending on platform, other open file descriptors may prevent immediate removal).

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### rename()

> `static` **rename**(`source`, `destination`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Renames (moves) a file from `source` to `destination`. Works across filesystems.

#### Parameters

##### source

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### destination

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### writeBytes()

> `static` **writeBytes**(`path`, `bytes`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Writes bytes to a file at the given path (static).

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### bytes

[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### writeText()

> `static` **writeText**(`path`, `text`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Writes text to a file at the given path (static).

```ts
await File.writeText("hello.txt", "Hello, world!");
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
