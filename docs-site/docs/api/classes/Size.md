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
println(a.add(b)); // "Size(15, 30)"
println(a.scale(2)); // "Size(20, 40)"
```

## Constructors

### Constructor

> **new Size**(`width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Size`

Constructor with two numbers.

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Size`

### Constructor

> **new Size**(`s`: [`SizeLike`](../type-aliases/SizeLike.md)): `Size`

Constructor with anything Size-like.

#### Parameters

##### s

[`SizeLike`](../type-aliases/SizeLike.md)

#### Returns

`Size`

## Properties

### width

> **width**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

width

***

### height

> **height**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

height

## Methods

### toJson()

> **toJson**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a JSON representation of this Size.

```ts
const s = new Size(100, 50);
println(s.toJson()); // '{"width":100,"height":50}'
```

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### equals()

> **equals**(`other`: `Size`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if a Size equals another.

```ts
const a = new Size(10, 20);
const b = new Size(10, 20);
println(a.equals(b)); // true
```

#### Parameters

##### other

`Size`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### add()

> **add**(`other`: `Size`): `Size`

Adds two sizes and returns a new Size.

```ts
const sum = new Size(10, 20).add(new Size(5, 10));
println(sum); // "Size(15, 30)"
```

#### Parameters

##### other

`Size`

#### Returns

`Size`

***

### subtract()

> **subtract**(`other`: `Size`): `Size`

Subtracts two sizes and returns a new Size.

```ts
const diff = new Size(100, 50).subtract(new Size(30, 20));
println(diff); // "Size(70, 30)"
```

#### Parameters

##### other

`Size`

#### Returns

`Size`

***

### scale()

> **scale**(`factor`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Size`

Scales this size by a factor and returns a new Size.

```ts
const s = new Size(10, 20).scale(3);
println(s); // "Size(30, 60)"
```

#### Parameters

##### factor

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Size`

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this size.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

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
