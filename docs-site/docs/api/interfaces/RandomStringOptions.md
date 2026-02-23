# Interface: RandomStringOptions


Options for generating random strings.

```ts
const token = random.string(32);
const pin = random.string(6, { characters: "0123456789" });
```

## Properties

### allowLetters?

> `optional` **allowLetters**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Include letters `A-Z` and `a-z` in the default character set.
Ignored when `characters` is specified.

#### Default Value

`true`

***

### allowNumbers?

> `optional` **allowNumbers**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Include digits `0-9` in the default character set.
Ignored when `characters` is specified.

#### Default Value

`true`

***

### allowSpecialCharacters?

> `optional` **allowSpecialCharacters**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Include printable ASCII non-alphanumeric characters in the default character set.
Ignored when `characters` is specified.

#### Default Value

`true`

***

### characters?

> `optional` **characters**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Possible characters to pick from.
Can contain any Unicode grapheme cluster.
When `characters` is specified, `allowNumbers`, `allowLetters` and `allowSpecialCharacters` are ignored.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) (all printable ASCII characters)
