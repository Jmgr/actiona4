# Interface: Web

Defined in: [index.d.ts:6769](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6769)

HTTP client for downloading files, text, images, and binary data.

Supports progress tracking, authentication, custom headers, and multipart uploads.

```ts
const text = await web.downloadText("https://example.com/data.json");
```

```ts
const image = await web.downloadImage("https://example.com/photo.png");
console.log(image.size().toString());
```

## Methods

### download()

> **download**(`url`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<`Uint8Array`, [`WebProgress`](WebProgress.md)\>

Defined in: [index.d.ts:6785](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6785)

Downloads a binary file.

```ts
const bytes = await web.download("https://example.com/file.bin");
```

```ts
const task = web.download("https://example.com/file.bin");
for await (const progress of task) {
console.log(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const bytes = await task;
```

#### Parameters

##### url

`string`

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<`Uint8Array`, [`WebProgress`](WebProgress.md)\>

***

### downloadFile()

> **downloadFile**(`url`, `directory?`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<`string`, [`WebProgress`](WebProgress.md)\>

Defined in: [index.d.ts:6833](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6833)

Downloads a file to a directory.

```ts
const filePath = await web.downloadFile("https://example.com/file.zip");
```

```ts
const task = web.downloadFile("https://example.com/file.zip", "/tmp");
for await (const progress of task) {
console.log(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const filePath = await task;
```

#### Parameters

##### url

`string`

##### directory?

`string`

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<`string`, [`WebProgress`](WebProgress.md)\>

***

### downloadImage()

> **downloadImage**(`url`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Image`](../classes/Image.md), [`WebProgress`](WebProgress.md)\>

Defined in: [index.d.ts:6817](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6817)

Downloads an image.

```ts
const image = await web.downloadImage("https://example.com/photo.png");
```

```ts
const task = web.downloadImage("https://example.com/photo.png");
for await (const progress of task) {
console.log(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const image = await task;
```

#### Parameters

##### url

`string`

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Image`](../classes/Image.md), [`WebProgress`](WebProgress.md)\>

***

### downloadText()

> **downloadText**(`url`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<`string`, [`WebProgress`](WebProgress.md)\>

Defined in: [index.d.ts:6801](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6801)

Downloads a text file.

```ts
const text = await web.downloadText("https://example.com/data.json");
```

```ts
const task = web.downloadText("https://example.com/data.json");
for await (const progress of task) {
console.log(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const text = await task;
```

#### Parameters

##### url

`string`

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<`string`, [`WebProgress`](WebProgress.md)\>
