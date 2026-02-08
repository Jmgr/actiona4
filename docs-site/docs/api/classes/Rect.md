# Class: Rect

Defined in: [index.d.ts:5240](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5240)

A 2D rectangle with position and size.

Rects can be constructed from four numbers, an object with `x`/`y`/`width`/`height`, or another Rect.

```ts
const r1 = new Rect(0, 0, 100, 50);
const r2 = new Rect({ x: 0, y: 0, width: 100, height: 50 });
```

```ts
const a = new Rect(0, 0, 100, 100);
const b = new Rect(50, 50, 100, 100);
console.log(a.intersects(b)); // true
const inter = a.intersection(b); // Rect(50, 50, 50, 50)
```

## Constructors

### Constructor

> **new Rect**(`x`, `y`, `width`, `height`): `Rect`

Defined in: [index.d.ts:5268](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5268)

Constructor with a position and a size.

#### Parameters

##### x

`number`

##### y

`number`

##### width

`number`

##### height

`number`

#### Returns

`Rect`

### Constructor

> **new Rect**(`r`): `Rect`

Defined in: [index.d.ts:5272](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5272)

Constructor with anything Rect-like.

#### Parameters

##### r

[`RectLike`](../type-aliases/RectLike.md)

#### Returns

`Rect`

## Properties

### height

> **height**: `number`

Defined in: [index.d.ts:5256](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5256)

Height

***

### size

> **size**: [`Size`](Size.md)

Defined in: [index.d.ts:5264](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5264)

Size

***

### topLeft

> **topLeft**: [`Point`](Point.md)

Defined in: [index.d.ts:5260](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5260)

Top-left origin

***

### width

> **width**: `number`

Defined in: [index.d.ts:5252](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5252)

Width

***

### x

> **x**: `number`

Defined in: [index.d.ts:5244](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5244)

X coordinate

***

### y

> **y**: `number`

Defined in: [index.d.ts:5248](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5248)

Y coordinate

## Methods

### clone()

> **clone**(): `Rect`

Defined in: [index.d.ts:5305](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5305)

Clones this Rect.

```ts
const original = new Rect(0, 0, 100, 100);
const copy = original.clone();
```

#### Returns

`Rect`

***

### contains()

> **contains**(`point`): `boolean`

Defined in: [index.d.ts:5292](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5292)

Returns true if this Rect contains the given point.

```ts
const r = new Rect(0, 0, 100, 100);
console.log(r.contains(new Point(50, 50)));  // true
console.log(r.contains(new Point(150, 50))); // false
```

#### Parameters

##### point

[`Point`](Point.md)

#### Returns

`boolean`

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:5282](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5282)

Returns true if this Rect equals another.

```ts
const a = new Rect(0, 0, 10, 10);
const b = new Rect(0, 0, 10, 10);
console.log(a.equals(b)); // true
```

#### Parameters

##### other

`Rect`

#### Returns

`boolean`

***

### intersection()

> **intersection**(`other`): `Rect` \| `undefined`

Defined in: [index.d.ts:5325](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5325)

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

`Rect` \| `undefined`

***

### intersects()

> **intersects**(`other`): `boolean`

Defined in: [index.d.ts:5315](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5315)

Returns true if this Rect intersects with another.

```ts
const a = new Rect(0, 0, 100, 100);
const b = new Rect(50, 50, 100, 100);
console.log(a.intersects(b)); // true
```

#### Parameters

##### other

`Rect`

#### Returns

`boolean`

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5296](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5296)

Returns a string representation of this Rect.

#### Returns

`string`

***

### union()

> **union**(`other`): `Rect`

Defined in: [index.d.ts:5335](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5335)

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
