# Class: Filesystem

Defined in: [index.d.ts:3739](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3739)

Provides static methods for querying filesystem path types.

```ts
if (await Filesystem.exists("/tmp/myfile.txt")) {
console.log("exists!");
}

if (await Filesystem.isFile("/tmp/myfile.txt")) {
console.log("it's a file");
} else if (await Filesystem.isDirectory("/tmp/myfile.txt")) {
console.log("it's a directory");
}
```

## Methods

### exists()

> `static` **exists**(`path`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:3744](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3744)

Returns `true` if a path exists on the filesystem.

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`boolean`\>

***

### isDirectory()

> `static` **isDirectory**(`path`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:3752](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3752)

Returns `true` if the path points to a directory.

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`boolean`\>

***

### isFile()

> `static` **isFile**(`path`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:3748](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3748)

Returns `true` if the path points to a regular file.

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`boolean`\>

***

### isSymlink()

> `static` **isSymlink**(`path`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:3756](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3756)

Returns `true` if the path points to a symbolic link.

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`boolean`\>
