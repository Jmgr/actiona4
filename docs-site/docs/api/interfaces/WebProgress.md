# Interface: WebProgress

Progress information for web downloads and uploads.

```ts
const task = web.download("https://example.com/file.bin");
for await (const progress of task) {
console.log(
formatBytes(progress.current),
formatBytes(progress.total),
progress.finished,
);
}
```

## Properties

### current

> `readonly` **current**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Bytes transferred so far.

***

### finished

> `readonly` **finished**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the transfer is complete.

***

### total

> `readonly` **total**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Total bytes expected (0 if unknown).
