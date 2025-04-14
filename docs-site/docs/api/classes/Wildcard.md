# Class: Wildcard

A wildcard pattern for matching strings.

Supports `*` (match any sequence) and `?` (match any single character).

```ts
const pattern = new Wildcard("*.txt");
```

```ts
// Used in APIs that accept a NameLike parameter
const pattern = new Wildcard("my_app*");
```

## Constructors

### Constructor

> **new Wildcard**(`pattern`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): `Wildcard`

Constructor.

#### Parameters

##### pattern

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

`Wildcard`

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this wildcard.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
