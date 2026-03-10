# Class: Filesystem

Provides static methods for querying filesystem path types.

```ts
if (await Filesystem.exists("/tmp/myfile.txt")) {
    println("exists!");
}

if (await Filesystem.isFile("/tmp/myfile.txt")) {
    println("it's a file");
} else if (await Filesystem.isDirectory("/tmp/myfile.txt")) {
    println("it's a directory");
}
```

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this filesystem.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### exists()

> <span class="async-badge">async</span> `static` **exists**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns `true` if a path exists on the filesystem.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

***

### isDirectory()

> <span class="async-badge">async</span> `static` **isDirectory**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns `true` if the path points to a directory.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

***

### isFile()

> <span class="async-badge">async</span> `static` **isFile**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns `true` if the path points to a regular file.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

***

### isSymlink()

> <span class="async-badge">async</span> `static` **isSymlink**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns `true` if the path points to a symbolic link.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>
