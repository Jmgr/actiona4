# Class: MultipartForm

Defined in: [index.d.ts:6635](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6635)

Multipart form for uploading files and data.

```ts
const form = new MultipartForm();
form.addText("title", "My Upload");
form.addFile("file", "/path/to/file.txt");
const result = await web.downloadText("https://example.com/upload", {
method: Method.Post,
multipart: form,
});
```

## Constructors

### Constructor

> **new MultipartForm**(): `MultipartForm`

Defined in: [index.d.ts:6636](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6636)

#### Returns

`MultipartForm`

## Methods

### addBytes()

> **addBytes**(`name`, `bytes`, `filename?`, `mimetype?`): `void`

Defined in: [index.d.ts:6664](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6664)

Adds a byte field.

```ts
const form = new MultipartForm();
const bytes = new Uint8Array([72, 101, 108, 108, 111]);
form.addBytes("data", bytes, "hello.bin");
```

#### Parameters

##### name

`string`

##### bytes

`Uint8Array`

##### filename?

`string`

##### mimetype?

`string`

#### Returns

`void`

***

### addFile()

> **addFile**(`name`, `path`, `filename?`, `mimetype?`): `void`

Defined in: [index.d.ts:6654](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6654)

Adds a file field.

```ts
const form = new MultipartForm();
form.addFile("document", "/path/to/report.pdf");
```

#### Parameters

##### name

`string`

##### path

`string`

##### filename?

`string`

##### mimetype?

`string`

#### Returns

`void`

***

### addText()

> **addText**(`name`, `value`, `filename?`, `mimetype?`): `void`

Defined in: [index.d.ts:6645](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6645)

Adds a text field.

```ts
const form = new MultipartForm();
form.addText("username", "john");
```

#### Parameters

##### name

`string`

##### value

`string`

##### filename?

`string`

##### mimetype?

`string`

#### Returns

`void`
