# Interface: DirectoryListOptions


Options for `Directory.listEntries()`.

```ts
const entries = await Directory.listEntries("/tmp", {
  sort: false,
  absolutePath: false,
  fetchSize: true,
});
```

## Properties

### sort?

> `optional` **sort?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the entries be sorted?

#### Default Value

`true`

***

### fetchSize?

> `optional` **fetchSize?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should each entry's size be fetched?

#### Default Value

`true`
