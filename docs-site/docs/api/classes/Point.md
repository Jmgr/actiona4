# Class: Point

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
println(a.distanceTo(b)); // 5
println(a.add(b)); // "Point(5, 8)"
```

## Constructors

### Constructor

> **new Point**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Point`

Constructor with two numbers.

#### Parameters

##### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Point`

### Constructor

> **new Point**(`p`: [`PointLike`](../type-aliases/PointLike.md)): `Point`

Constructor with anything Point-like.

#### Parameters

##### p

[`PointLike`](../type-aliases/PointLike.md)

#### Returns

`Point`

## Properties

### x

> **x**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

X coordinate

***

### y

> **y**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Y coordinate

***

### Zero

> `readonly` `static` **Zero**: `Point`

Point(0, 0)

## Methods

### add()

> **add**(`other`: `Point`): `Point`

Adds two points and returns a new Point.

```ts
const sum = new Point(1, 2).add(new Point(3, 4));
println(sum); // "Point(4, 6)"
```

#### Parameters

##### other

`Point`

#### Returns

`Point`

***

### clone()

> **clone**(): `Point`

Clones this Point.

```ts
const original = new Point(1, 2);
const copy = original.clone();
```

#### Returns

`Point`

***

### distanceTo()

> **distanceTo**(`other`: `Point`): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Calculates the distance between this point and another.

```ts
const a = new Point(0, 0);
const b = new Point(3, 4);
println(a.distanceTo(b)); // 5
```

#### Parameters

##### other

`Point`

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### equals()

> **equals**(`other`: `Point`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if a Point equals another.

```ts
const a = new Point(1, 2);
const b = new Point(1, 2);
println(a.equals(b)); // true
```

#### Parameters

##### other

`Point`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### isOrigin()

> **isOrigin**(): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if this Point is at the origin, (0, 0).

```ts
println(new Point(0, 0).isOrigin()); // true
println(new Point(1, 0).isOrigin()); // false
```

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### length()

> **length**(): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Length of this point (distance from origin).

```ts
const p = new Point(3, 4);
println(p.length()); // 5
```

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### scaled()

> **scaled**(`factor`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Point`

Scales this point by a factor and returns a new Point.

```ts
const p = new Point(3, 4).scaled(2);
println(p); // "Point(6, 8)"
```

#### Parameters

##### factor

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Point`

***

### subtract()

> **subtract**(`other`: `Point`): `Point`

Subtracts two points and returns a new Point.

```ts
const diff = new Point(5, 7).subtract(new Point(2, 3));
println(diff); // "Point(3, 4)"
```

#### Parameters

##### other

`Point`

#### Returns

`Point`

***

### toJson()

> **toJson**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a JSON representation of this Point.

```ts
const p = new Point(1, 2);
println(p.toJson()); // '{"x":1,"y":2}'
```

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this Point.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### distance()

> `static` **distance**(`a`: `Point`, `b`: `Point`): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Computes the distance between two points.

```ts
const d = Point.distance(new Point(0, 0), new Point(3, 4));
println(d); // 5
```

#### Parameters

##### a

`Point`

##### b

`Point`

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### randomInCircle()

#### Call Signature

> `static` **randomInCircle**(`center`: [`PointLike`](../type-aliases/PointLike.md), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Point`

Returns a random point within a circle of the given radius around a center point.

```ts
const p = Point.randomInCircle(100, 100, 50);
```

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Point`

#### Call Signature

> `static` **randomInCircle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Point`

Returns a random point within a circle of the given radius around a center point.

```ts
const p = Point.randomInCircle(100, 100, 50);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Point`
