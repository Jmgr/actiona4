# Class: Size

A 2D size with width and height.

Sizes can be constructed from two numbers, an object with `width`/`height`, or another Size.

```ts
const s1 = new Size(100, 50);
const s2 = new Size({ width: 100, height: 50 });
const s3 = new Size(s1);
```

```ts
const a = new Size(10, 20);
const b = new Size(5, 10);
console.log(a.add(b).toString()); // "(15, 30)"
console.log(a.scale(2).toString()); // "(20, 40)"
```

## Constructors

### Constructor

> **new Size**(`width`, `height`): `Size`

Constructor with two numbers.

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Size`

### Constructor

> **new Size**(`s`): `Size`

Constructor with anything Size-like.

#### Parameters

##### s

[`SizeLike`](../type-aliases/SizeLike.md)

#### Returns

`Size`

## Properties

### height

> **height**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

height

***

### width

> **width**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

width

## Methods

### add()

> **add**(`other`): `Size`

Adds two sizes and returns a new Size.

```ts
const sum = new Size(10, 20).add(new Size(5, 10));
console.log(sum.toString()); // "(15, 30)"
```

#### Parameters

##### other

`Size`

#### Returns

`Size`

***

### clone()

> **clone**(): `Size`

Clones this Size.

```ts
const original = new Size(100, 50);
const copy = original.clone();
```

#### Returns

`Size`

***

### equals()

> **equals**(`other`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if a Size equals another.

```ts
const a = new Size(10, 20);
const b = new Size(10, 20);
console.log(a.equals(b)); // true
```

#### Parameters

##### other

`Size`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### scale()

> **scale**(`factor`): `Size`

Scales this size by a factor and returns a new Size.

```ts
const s = new Size(10, 20).scale(3);
console.log(s.toString()); // "(30, 60)"
```

#### Parameters

##### factor

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Size`

***

### subtract()

> **subtract**(`other`): `Size`

Subtracts two sizes and returns a new Size.

```ts
const diff = new Size(100, 50).subtract(new Size(30, 20));
console.log(diff.toString()); // "(70, 30)"
```

#### Parameters

##### other

`Size`

#### Returns

`Size`

***

### toJson()

> **toJson**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a JSON representation of this Size.

```ts
const s = new Size(100, 50);
console.log(s.toJson()); // '{"width":100,"height":50}'
```

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this Size.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
