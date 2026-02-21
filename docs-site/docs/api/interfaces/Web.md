# Interface: Web

HTTP client for downloading files, text, images, and binary data.

Supports progress tracking, authentication, custom headers, and multipart uploads.

```ts
const text = await web.downloadText("https://example.com/data.json");
```

```ts
const image = await web.downloadImage("https://example.com/photo.png");
println(image.size().toString());
```

## Methods

### download()

> **download**(`url`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array), [`WebProgress`](WebProgress.md)\>

Downloads a binary file.

```ts
const bytes = await web.download("https://example.com/file.bin");
```

```ts
const task = web.download("https://example.com/file.bin");
for await (const progress of task) {
  println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const bytes = await task;
```

#### Parameters

##### url

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array), [`WebProgress`](WebProgress.md)\>

***

### downloadFile()

> **downloadFile**(`url`, `directory?`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

Downloads a file to a directory.

```ts
const filePath = await web.downloadFile("https://example.com/file.zip");
```

```ts
const task = web.downloadFile("https://example.com/file.zip", "/tmp");
for await (const progress of task) {
  println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const filePath = await task;
```

#### Parameters

##### url

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### directory?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

***

### downloadImage()

> **downloadImage**(`url`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Image`](../classes/Image.md), [`WebProgress`](WebProgress.md)\>

Downloads an image.

```ts
const image = await web.downloadImage("https://example.com/photo.png");
```

```ts
const task = web.downloadImage("https://example.com/photo.png");
for await (const progress of task) {
  println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const image = await task;
```

#### Parameters

##### url

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Image`](../classes/Image.md), [`WebProgress`](WebProgress.md)\>

***

### downloadText()

> **downloadText**(`url`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

Downloads a text file.

```ts
const text = await web.downloadText("https://example.com/data.json");
```

```ts
const task = web.downloadText("https://example.com/data.json");
for await (const progress of task) {
  println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
}
const text = await task;
```

#### Parameters

##### url

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`WebOptions`](WebOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>
