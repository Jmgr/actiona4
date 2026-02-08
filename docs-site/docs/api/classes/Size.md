# Class: Size

Defined in: [index.d.ts:5517](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5517)

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

Defined in: [index.d.ts:5529](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5529)

Constructor with two numbers.

#### Parameters

##### width

`number`

##### height

`number`

#### Returns

`Size`

### Constructor

> **new Size**(`s`): `Size`

Defined in: [index.d.ts:5533](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5533)

Constructor with anything Size-like.

#### Parameters

##### s

[`SizeLike`](../type-aliases/SizeLike.md)

#### Returns

`Size`

## Properties

### height

> **height**: `number`

Defined in: [index.d.ts:5525](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5525)

height

***

### width

> **width**: `number`

Defined in: [index.d.ts:5521](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5521)

width

## Methods

### add()

> **add**(`other`): `Size`

Defined in: [index.d.ts:5561](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5561)

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

Defined in: [index.d.ts:5592](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5592)

Clones this Size.

```ts
const original = new Size(100, 50);
const copy = original.clone();
```

#### Returns

`Size`

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:5552](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5552)

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

`boolean`

***

### scale()

> **scale**(`factor`): `Size`

Defined in: [index.d.ts:5579](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5579)

Scales this size by a factor and returns a new Size.

```ts
const s = new Size(10, 20).scale(3);
console.log(s.toString()); // "(30, 60)"
```

#### Parameters

##### factor

`number`

#### Returns

`Size`

***

### subtract()

> **subtract**(`other`): `Size`

Defined in: [index.d.ts:5570](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5570)

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

> **toJson**(): `string`

Defined in: [index.d.ts:5542](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5542)

Returns a JSON representation of this Size.

```ts
const s = new Size(100, 50);
console.log(s.toJson()); // '{"width":100,"height":50}'
```

#### Returns

`string`

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5583](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5583)

Returns a string representation of this Size.

#### Returns

`string`
