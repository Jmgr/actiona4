# Class: Point

Defined in: [index.d.ts:5005](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5005)

A 2D point with integer coordinates.

Points can be constructed from two numbers, an object with `x`/`y`, or another Point.

```ts
const p1 = new Point(10, 20);
const p2 = new Point({ x: 10, y: 20 });
const p3 = new Point(p1);
```

```ts
const a = new Point(1, 2);
const b = new Point(4, 6);
console.log(a.distanceTo(b)); // 5
console.log(a.add(b).toString()); // "(5, 8)"
```

## Constructors

### Constructor

> **new Point**(`x`, `y`): `Point`

Defined in: [index.d.ts:5017](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5017)

Constructor with two numbers.

#### Parameters

##### x

`number`

##### y

`number`

#### Returns

`Point`

### Constructor

> **new Point**(`p`): `Point`

Defined in: [index.d.ts:5021](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5021)

Constructor with anything Point-like.

#### Parameters

##### p

[`PointLike`](../type-aliases/PointLike.md)

#### Returns

`Point`

## Properties

### x

> **x**: `number`

Defined in: [index.d.ts:5009](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5009)

X coordinate

***

### y

> **y**: `number`

Defined in: [index.d.ts:5013](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5013)

Y coordinate

## Methods

### add()

> **add**(`other`): `Point`

Defined in: [index.d.ts:5102](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5102)

Adds two points and returns a new Point.

```ts
const sum = new Point(1, 2).add(new Point(3, 4));
console.log(sum.toString()); // "(4, 6)"
```

#### Parameters

##### other

`Point`

#### Returns

`Point`

***

### clone()

> **clone**(): `Point`

Defined in: [index.d.ts:5133](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5133)

Clones this Point.

```ts
const original = new Point(1, 2);
const copy = original.clone();
```

#### Returns

`Point`

***

### distanceTo()

> **distanceTo**(`other`): `number`

Defined in: [index.d.ts:5056](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5056)

Calculates the distance between this point and another.

```ts
const a = new Point(0, 0);
const b = new Point(3, 4);
console.log(a.distanceTo(b)); // 5
```

#### Parameters

##### other

`Point`

#### Returns

`number`

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:5093](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5093)

Returns true if a Point equals another.

```ts
const a = new Point(1, 2);
const b = new Point(1, 2);
console.log(a.equals(b)); // true
```

#### Parameters

##### other

`Point`

#### Returns

`boolean`

***

### isOrigin()

> **isOrigin**(): `boolean`

Defined in: [index.d.ts:5074](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5074)

Returns true if this Point is at the origin, (0, 0).

```ts
console.log(new Point(0, 0).isOrigin()); // true
console.log(new Point(1, 0).isOrigin()); // false
```

#### Returns

`boolean`

***

### length()

> **length**(): `number`

Defined in: [index.d.ts:5030](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5030)

Length of this point (distance from origin).

```ts
const p = new Point(3, 4);
console.log(p.length()); // 5
```

#### Returns

`number`

***

### scaled()

> **scaled**(`factor`): `Point`

Defined in: [index.d.ts:5120](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5120)

Scales this point by a factor and returns a new Point.

```ts
const p = new Point(3, 4).scaled(2);
console.log(p.toString()); // "(6, 8)"
```

#### Parameters

##### factor

`number`

#### Returns

`Point`

***

### subtract()

> **subtract**(`other`): `Point`

Defined in: [index.d.ts:5111](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5111)

Subtracts two points and returns a new Point.

```ts
const diff = new Point(5, 7).subtract(new Point(2, 3));
console.log(diff.toString()); // "(3, 4)"
```

#### Parameters

##### other

`Point`

#### Returns

`Point`

***

### toJson()

> **toJson**(): `string`

Defined in: [index.d.ts:5065](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5065)

Returns a JSON representation of this Point.

```ts
const p = new Point(1, 2);
console.log(p.toJson()); // '{"x":1,"y":2}'
```

#### Returns

`string`

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5124](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5124)

Returns a string representation of this Point.

#### Returns

`string`

***

### distance()

> `static` **distance**(`a`, `b`): `number`

Defined in: [index.d.ts:5083](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5083)

Computes the distance between two points.

```ts
const d = Point.distance(new Point(0, 0), new Point(3, 4));
console.log(d); // 5
```

#### Parameters

##### a

`Point`

##### b

`Point`

#### Returns

`number`

***

### randomInCircle()

#### Call Signature

> `static` **randomInCircle**(`center`, `radius`): `Point`

Defined in: [index.d.ts:5038](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5038)

Returns a random point within a circle of the given radius around a center point.

```ts
const p = Point.randomInCircle(100, 100, 50);
```

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

`number`

##### Returns

`Point`

#### Call Signature

> `static` **randomInCircle**(`x`, `y`, `radius`): `Point`

Defined in: [index.d.ts:5046](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5046)

Returns a random point within a circle of the given radius around a center point.

```ts
const p = Point.randomInCircle(100, 100, 50);
```

##### Parameters

###### x

`number`

###### y

`number`

###### radius

`number`

##### Returns

`Point`
