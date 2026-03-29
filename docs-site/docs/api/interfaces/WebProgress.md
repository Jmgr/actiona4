# Interface: WebProgress

Progress information for web downloads and uploads.

```ts
const task = web.download("https://example.com/file.bin");
for await (const progress of task) {
  println(
    formatBytes(progress.current),
    formatBytes(progress.total),
    progress.finished,
  );
}
```

## Properties

### total

> `readonly` **total**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total bytes expected (0 if unknown).

***

### current

> `readonly` **current**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Bytes transferred so far.

***

### finished

> `readonly` **finished**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the transfer is complete.

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this web transfer progress.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
