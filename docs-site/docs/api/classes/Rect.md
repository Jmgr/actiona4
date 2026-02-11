# Class: Rect

A 2D rectangle with position and size.

Rects can be constructed from four numbers, an object with `x`/`y`/`width`/`height`, or another Rect.

```ts
const r1 = new Rect(0, 0, 100, 50);
const r2 = new Rect({ x: 0, y: 0, width: 100, height: 50 });
```

```ts
const a = new Rect(0, 0, 100, 100);
const b = new Rect(50, 50, 100, 100);
println(a.intersects(b)); // true
const inter = a.intersection(b); // Rect(50, 50, 50, 50)
```

## Constructors

### Constructor

> **new Rect**(`x`, `y`, `width`, `height`): `Rect`

Constructor with a position and a size.

#### Parameters

##### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Rect`

### Constructor

> **new Rect**(`r`): `Rect`

Constructor with anything Rect-like.

#### Parameters

##### r

[`RectLike`](../type-aliases/RectLike.md)

#### Returns

`Rect`

## Properties

### height

> **height**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Height

***

### size

> **size**: [`Size`](Size.md)

Size

***

### topLeft

> **topLeft**: [`Point`](Point.md)

Top-left origin

***

### width

> **width**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Width

***

### x

> **x**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

X coordinate

***

### y

> **y**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Y coordinate

## Methods

### clone()

> **clone**(): `Rect`

Clones this Rect.

```ts
const original = new Rect(0, 0, 100, 100);
const copy = original.clone();
```

#### Returns

`Rect`

***

### contains()

> **contains**(`point`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if this Rect contains the given point.

```ts
const r = new Rect(0, 0, 100, 100);
println(r.contains(new Point(50, 50)));  // true
println(r.contains(new Point(150, 50))); // false
```

#### Parameters

##### point

[`Point`](Point.md)

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### equals()

> **equals**(`other`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if this Rect equals another.

```ts
const a = new Rect(0, 0, 10, 10);
const b = new Rect(0, 0, 10, 10);
println(a.equals(b)); // true
```

#### Parameters

##### other

`Rect`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### intersection()

> **intersection**(`other`): `Rect` \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

Returns the intersection of two Rects, or undefined if they don't overlap.

```ts
const a = new Rect(0, 0, 100, 100);
const b = new Rect(50, 50, 100, 100);
const inter = a.intersection(b); // Rect(50, 50, 50, 50)
```

#### Parameters

##### other

`Rect`

#### Returns

`Rect` \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### intersects()

> **intersects**(`other`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if this Rect intersects with another.

```ts
const a = new Rect(0, 0, 100, 100);
const b = new Rect(50, 50, 100, 100);
println(a.intersects(b)); // true
```

#### Parameters

##### other

`Rect`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this Rect.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### union()

> **union**(`other`): `Rect`

Returns the smallest Rect containing both this and another Rect.

```ts
const a = new Rect(0, 0, 50, 50);
const b = new Rect(25, 25, 50, 50);
const u = a.union(b); // Rect(0, 0, 75, 75)
```

#### Parameters

##### other

`Rect`

#### Returns

`Rect`
