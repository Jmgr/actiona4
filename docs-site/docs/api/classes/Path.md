# Class: Path

Defined in: [index.d.ts:4911](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4911)

Utilities for manipulating file paths. All methods are static.

```ts
const full = Path.join("/home/user", "documents", "file.txt");
const dir = Path.parent(full);   // "/home/user/documents"
const name = Path.filename(full); // "file.txt"
const ext = Path.extension(full); // "txt"
```

```ts
// Change a file's extension
const newPath = Path.setExtension("/tmp/data.csv", "json");
// "/tmp/data.json"
```

## Methods

### basename()

> `static` **basename**(`path`): `string`

Defined in: [index.d.ts:4932](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4932)

Alias for `filename`.

#### Parameters

##### path

`string`

#### Returns

`string`

***

### dirname()

> `static` **dirname**(`path`): `string`

Defined in: [index.d.ts:4944](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4944)

Alias for `parent`.

#### Parameters

##### path

`string`

#### Returns

`string`

***

### extension()

> `static` **extension**(`path`): `string`

Defined in: [index.d.ts:4971](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4971)

Returns the file extension of a path (without the leading dot).

```ts
Path.extension("/home/user/file.txt"); // "txt"
Path.extension("/home/user/file"); // ""
```

#### Parameters

##### path

`string`

#### Returns

`string`

***

### extname()

> `static` **extname**(`path`): `string`

Defined in: [index.d.ts:4975](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4975)

Alias for `extension`.

#### Parameters

##### path

`string`

#### Returns

`string`

***

### filename()

> `static` **filename**(`path`): `string`

Defined in: [index.d.ts:4928](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4928)

Returns the file name component of a path.

```ts
Path.filename("/home/user/file.txt"); // "file.txt"
```

#### Parameters

##### path

`string`

#### Returns

`string`

***

### isAbsolute()

> `static` **isAbsolute**(`path`): `boolean`

Defined in: [index.d.ts:4953](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4953)

Returns whether the path is absolute.

```ts
Path.isAbsolute("/home/user"); // true
Path.isAbsolute("relative/path"); // false
```

#### Parameters

##### path

`string`

#### Returns

`boolean`

***

### isRelative()

> `static` **isRelative**(`path`): `boolean`

Defined in: [index.d.ts:4962](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4962)

Returns whether the path is relative.

```ts
Path.isRelative("relative/path"); // true
Path.isRelative("/absolute/path"); // false
```

#### Parameters

##### path

`string`

#### Returns

`boolean`

***

### join()

> `static` **join**(...`args`): `string`

Defined in: [index.d.ts:4920](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4920)

Joins path segments into a single path.

```ts
Path.join("/home", "user", "file.txt"); // "/home/user/file.txt"
```

#### Parameters

##### args

...`string`[]

#### Returns

`string`

***

### parent()

> `static` **parent**(`path`): `string`

Defined in: [index.d.ts:4940](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4940)

Returns the parent directory of a path.

```ts
Path.parent("/home/user/file.txt"); // "/home/user"
```

#### Parameters

##### path

`string`

#### Returns

`string`

***

### setExtension()

> `static` **setExtension**(`path`, `extension`): `string`

Defined in: [index.d.ts:4984](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4984)

Returns the path with a different extension. Returns an empty string on failure.

```ts
Path.setExtension("/tmp/data.csv", "json"); // "/tmp/data.json"
Path.setExtension("/tmp/archive.tar.gz", "xz"); // "/tmp/archive.tar.xz"
```

#### Parameters

##### path

`string`

##### extension

`string`

#### Returns

`string`
