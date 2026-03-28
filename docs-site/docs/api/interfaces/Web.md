# Interface: Web

HTTP client for downloading files, text, images, and binary data.

Supports progress tracking, authentication, custom headers, and multipart uploads.

```ts
const text = await web.downloadText("https://example.com/data.json");
```

```ts
const image = await web.downloadImage("https://example.com/photo.png");
println(image.size());
```

## Methods

### download()

> <span class="async-badge">async</span> **download**(`url`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`WebOptions`](WebOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array), [`WebProgress`](WebProgress.md)\>

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

<div class="options-fields">

###### contentType?

> `optional` **contentType?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Sets the content-type header.
Overrides any content-type set by other fields.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### form?

> `optional` **form?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Form data as strings.
Sets content-type to "application/x-www-form-urlencoded".

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### headers?

> `optional` **headers?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional HTTP headers to send with the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### method?

> `optional` **method?**: [`Method`](../enumerations/Method.md)

<div class="options-fields">

###### Connect

> **Connect**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Connect`

***

###### Delete

> **Delete**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Delete`

***

###### Get

> **Get**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Get`

***

###### Head

> **Head**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Head`

***

###### Options

> **Options**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Options`

***

###### Patch

> **Patch**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Patch`

***

###### Post

> **Post**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Post`

***

###### Put

> **Put**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Put`

***

###### Trace

> **Trace**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Trace`

</div>

HTTP method to use for the request.

###### Default Value

`Method.Get`

***

###### multipart?

> `optional` **multipart?**: [`MultipartForm`](../classes/MultipartForm.md)

Form multipart data.
Sets content-type and content-length appropriately.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### password?

> `optional` **password?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Password for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### query?

> `optional` **query?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional query parameters.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Request timeout duration.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### userName?

> `optional` **userName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

User name for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array), [`WebProgress`](WebProgress.md)\>

***

### downloadFile()

> <span class="async-badge">async</span> **downloadFile**(`url`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `directory?`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`WebOptions`](WebOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

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

<div class="options-fields">

###### contentType?

> `optional` **contentType?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Sets the content-type header.
Overrides any content-type set by other fields.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### form?

> `optional` **form?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Form data as strings.
Sets content-type to "application/x-www-form-urlencoded".

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### headers?

> `optional` **headers?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional HTTP headers to send with the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### method?

> `optional` **method?**: [`Method`](../enumerations/Method.md)

<div class="options-fields">

###### Connect

> **Connect**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Connect`

***

###### Delete

> **Delete**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Delete`

***

###### Get

> **Get**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Get`

***

###### Head

> **Head**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Head`

***

###### Options

> **Options**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Options`

***

###### Patch

> **Patch**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Patch`

***

###### Post

> **Post**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Post`

***

###### Put

> **Put**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Put`

***

###### Trace

> **Trace**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Trace`

</div>

HTTP method to use for the request.

###### Default Value

`Method.Get`

***

###### multipart?

> `optional` **multipart?**: [`MultipartForm`](../classes/MultipartForm.md)

Form multipart data.
Sets content-type and content-length appropriately.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### password?

> `optional` **password?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Password for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### query?

> `optional` **query?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional query parameters.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Request timeout duration.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### userName?

> `optional` **userName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

User name for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

***

### downloadImage()

> <span class="async-badge">async</span> **downloadImage**(`url`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`WebOptions`](WebOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Image`](../classes/Image.md), [`WebProgress`](WebProgress.md)\>

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

<div class="options-fields">

###### contentType?

> `optional` **contentType?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Sets the content-type header.
Overrides any content-type set by other fields.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### form?

> `optional` **form?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Form data as strings.
Sets content-type to "application/x-www-form-urlencoded".

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### headers?

> `optional` **headers?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional HTTP headers to send with the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### method?

> `optional` **method?**: [`Method`](../enumerations/Method.md)

<div class="options-fields">

###### Connect

> **Connect**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Connect`

***

###### Delete

> **Delete**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Delete`

***

###### Get

> **Get**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Get`

***

###### Head

> **Head**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Head`

***

###### Options

> **Options**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Options`

***

###### Patch

> **Patch**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Patch`

***

###### Post

> **Post**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Post`

***

###### Put

> **Put**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Put`

***

###### Trace

> **Trace**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Trace`

</div>

HTTP method to use for the request.

###### Default Value

`Method.Get`

***

###### multipart?

> `optional` **multipart?**: [`MultipartForm`](../classes/MultipartForm.md)

Form multipart data.
Sets content-type and content-length appropriately.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### password?

> `optional` **password?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Password for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### query?

> `optional` **query?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional query parameters.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Request timeout duration.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### userName?

> `optional` **userName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

User name for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Image`](../classes/Image.md), [`WebProgress`](WebProgress.md)\>

***

### downloadText()

> <span class="async-badge">async</span> **downloadText**(`url`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`WebOptions`](WebOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

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

<div class="options-fields">

###### contentType?

> `optional` **contentType?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Sets the content-type header.
Overrides any content-type set by other fields.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### form?

> `optional` **form?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Form data as strings.
Sets content-type to "application/x-www-form-urlencoded".

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### headers?

> `optional` **headers?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional HTTP headers to send with the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### method?

> `optional` **method?**: [`Method`](../enumerations/Method.md)

<div class="options-fields">

###### Connect

> **Connect**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Connect`

***

###### Delete

> **Delete**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Delete`

***

###### Get

> **Get**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Get`

***

###### Head

> **Head**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Head`

***

###### Options

> **Options**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Options`

***

###### Patch

> **Patch**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Patch`

***

###### Post

> **Post**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Post`

***

###### Put

> **Put**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Put`

***

###### Trace

> **Trace**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Method.Trace`

</div>

HTTP method to use for the request.

###### Default Value

`Method.Get`

***

###### multipart?

> `optional` **multipart?**: [`MultipartForm`](../classes/MultipartForm.md)

Form multipart data.
Sets content-type and content-length appropriately.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### password?

> `optional` **password?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Password for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### query?

> `optional` **query?**: [`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Additional query parameters.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the request.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### timeout?

> `optional` **timeout?**: [`DurationLike`](../type-aliases/DurationLike.md)

Request timeout duration.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### userName?

> `optional` **userName?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

User name for HTTP basic authentication.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`WebProgress`](WebProgress.md)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `web` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
