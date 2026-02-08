# Interface: DirectoryOptions

Defined in: [index.d.ts:3242](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3242)

Options for `Directory.create()` and `Directory.remove()`.

```ts
await Directory.create("/tmp/a/b/c", { recursive: true });
await Directory.remove("/tmp/a", { recursive: false });
```

## Properties

### recursive?

> `optional` **recursive**: `boolean`

Defined in: [index.d.ts:3247](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3247)

Should the directories be created or removed recursively?

#### Default Value

`true`
