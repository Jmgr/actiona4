# Interface: Random

Defined in: [index.d.ts:5154](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5154)

Random number generator.

Provides methods for generating random numbers, integers, positions, and choices.
The generator is deterministic when seeded.

```ts
const n = random.number(); // 0..1
const i = random.integer(1, 10); // 1..10
const item = random.choice(["a", "b", "c"]);
```

```ts
random.setSeed(42);
console.log(random.number()); // always the same value
random.resetSeed();
```

## Methods

### choice()

> **choice**\<`T`\>(`array`, `fallback?`): `T`

Defined in: [index.d.ts:5216](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5216)

Chooses one random entry in an array.
A fallback can be provided, in case the array is empty.

```ts
const item = random.choice(["apple", "banana", "cherry"]);
```

```ts
const item = random.choice([], "default");
console.log(item); // "default"
```

#### Type Parameters

##### T

`T`

#### Parameters

##### array

`T`[]

##### fallback?

`T`

#### Returns

`T`

***

### integer()

#### Call Signature

> **integer**(`max`): `number`

Defined in: [index.d.ts:5170](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5170)

Returns an integer between 0 (inclusive) and max (inclusive)

##### Parameters

###### max

`number`

##### Returns

`number`

#### Call Signature

> **integer**(`min`, `max`): `number`

Defined in: [index.d.ts:5174](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5174)

Returns an integer between min (inclusive) and max (inclusive)

##### Parameters

###### min

`number`

###### max

`number`

##### Returns

`number`

***

### number()

#### Call Signature

> **number**(): `number`

Defined in: [index.d.ts:5158](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5158)

Returns a number between 0 (inclusive) and 1 (exclusive)

##### Returns

`number`

#### Call Signature

> **number**(`max`): `number`

Defined in: [index.d.ts:5162](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5162)

Returns a number between 0 (inclusive) and max (exclusive)

##### Parameters

###### max

`number`

##### Returns

`number`

#### Call Signature

> **number**(`min`, `max`): `number`

Defined in: [index.d.ts:5166](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5166)

Returns a number between min (inclusive) and max (exclusive)

##### Parameters

###### min

`number`

###### max

`number`

##### Returns

`number`

***

### position()

> **position**(): `Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

Defined in: [index.d.ts:5202](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5202)

Returns a random position on any display.

```ts
const pos = await random.position();
console.log(pos.toString());
```

#### Returns

`Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

***

### resetSeed()

> **resetSeed**(): `void`

Defined in: [index.d.ts:5193](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5193)

Resets the seed to be a random one.

```ts
random.resetSeed();
```

#### Returns

`void`

***

### setSeed()

> **setSeed**(`seed`): `void`

Defined in: [index.d.ts:5185](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5185)

Sets the seed to a value.
This seed is used for all random number generation. Since the random number generator is
deterministic that means that setting it to a particular number will always generate the same
random numbers.

```ts
random.setSeed(42);
```

#### Parameters

##### seed

`number`

#### Returns

`void`
