# Interface: StandardPaths

Platform-specific standard directory paths.

All properties return the path as a string, or undefined if unavailable.

```ts
console.log(standardPaths.home);       // e.g. "/home/user"
console.log(standardPaths.downloads);   // e.g. "/home/user/Downloads"
console.log(standardPaths.documents);   // e.g. "/home/user/Documents"
```

## Properties

### cache?

> `readonly` `optional` **cache**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Cache directory

***

### config?

> `readonly` `optional` **config**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Config directory

***

### desktop?

> `readonly` `optional` **desktop**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Desktop directory

***

### documents?

> `readonly` `optional` **documents**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Documents directory

***

### downloads?

> `readonly` `optional` **downloads**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Downloads directory

***

### home?

> `readonly` `optional` **home**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Home directory

***

### localConfig?

> `readonly` `optional` **localConfig**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Local config directory

***

### music?

> `readonly` `optional` **music**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Music directory

***

### pictures?

> `readonly` `optional` **pictures**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Pictures directory

***

### public?

> `readonly` `optional` **public**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Public directory

***

### videos?

> `readonly` `optional` **videos**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Videos directory

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of all standard paths.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
