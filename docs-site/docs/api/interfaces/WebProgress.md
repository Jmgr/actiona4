# Interface: WebProgress

Defined in: [index.d.ts:6740](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6740)

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

> `readonly` **current**: `number`

Defined in: [index.d.ts:6748](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6748)

Bytes transferred so far.

***

### finished

> `readonly` **finished**: `boolean`

Defined in: [index.d.ts:6752](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6752)

Whether the transfer is complete.

***

### total

> `readonly` **total**: `number`

Defined in: [index.d.ts:6744](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6744)

Total bytes expected (0 if unknown).
