# Class: Wildcard

Defined in: [index.d.ts:4888](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4888)

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

> **new Wildcard**(`pattern`): `Wildcard`

Defined in: [index.d.ts:4892](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4892)

Constructor.

#### Parameters

##### pattern

`string`

#### Returns

`Wildcard`
