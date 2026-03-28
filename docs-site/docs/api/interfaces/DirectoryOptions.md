# Interface: DirectoryOptions


Options for `Directory.create()` and `Directory.remove()`.

```ts
await Directory.create("/tmp/a/b/c", { recursive: true });
await Directory.remove("/tmp/a", { recursive: false });
```

## Properties

### recursive?

> `optional` **recursive?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the directories be created or removed recursively?

#### Default Value

`true`
