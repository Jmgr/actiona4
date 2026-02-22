# Interface: OpenOptions


Options for `File.open()`.

```ts
// Read-only (default)
const file = await File.open("data.txt");

// Create a new file for writing
const file = await File.open("out.txt", {
    write: true,
    createNew: true,
});

// Append to an existing file
const file = await File.open("log.txt", {
    write: true,
    append: true,
});
```

## Properties

### append?

> `optional` **append**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Writing: open the file in append mode.
Note that setting this to `true` implies setting `write` to `true`.

#### Default Value

`false`

***

### create?

> `optional` **create**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Writing: create a new file if it doesn't exist.
Note that this only works if `write` or `append` are set to `true`.

#### Default Value

`false`

***

### createNew?

> `optional` **createNew**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Writing: always create a new file, even if one already exists.
Note that this only works if `write` or `append` are set to `true`.
Note that `create` and `truncate` are ignored if this is set to `true`.

#### Default Value

`false`

***

### read?

> `optional` **read**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the file be opened with read access?

#### Default Value

`true`

***

### truncate?

> `optional` **truncate**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Writing: truncate (remove all contents of) the file.
Note that this only works if `write` is `true`.

#### Default Value

`false`

***

### write?

> `optional` **write**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the file be opened with write access?

#### Default Value

`false`
