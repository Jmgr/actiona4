# Interface: Random

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
println(random.number()); // always the same value
random.resetSeed();
```

## Methods

### choice()

> **choice**\<`T`\>(`array`: `T`[], `fallback?`: `T`): `T`

Chooses one random entry in an array.
A fallback can be provided, in case the array is empty.

```ts
const item = random.choice(["apple", "banana", "cherry"]);
```

```ts
const item = random.choice([], "default");
println(item); // "default"
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

### color()

> **color**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Color`](../classes/Color.md)\>

Returns a random color with full opacity.

```ts
const c = random.color();
println(c); // Color(r: ?, g: ?, b: ?, a: 255)
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Color`](../classes/Color.md)\>

***

### colorWithAlpha()

> **colorWithAlpha**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Color`](../classes/Color.md)\>

Returns a random color including a random alpha channel.

```ts
const c = random.colorWithAlpha();
println(c); // Color(r: ?, g: ?, b: ?, a: ?)
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Color`](../classes/Color.md)\>

***

### integer()

#### Call Signature

> **integer**(`max`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns an integer between 0 (inclusive) and max (inclusive)

##### Parameters

###### max

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Call Signature

> **integer**(`min`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `max`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns an integer between min (inclusive) and max (inclusive)

##### Parameters

###### min

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### max

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### number()

#### Call Signature

> **number**(): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns a number between 0 (inclusive) and 1 (exclusive)

##### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Call Signature

> **number**(`max`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns a number between 0 (inclusive) and max (exclusive)

##### Parameters

###### max

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Call Signature

> **number**(`min`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `max`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns a number between min (inclusive) and max (exclusive)

##### Parameters

###### min

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### max

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### position()

> <span class="async-badge">async</span> **position**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

Returns a random position on any display.

```ts
const pos = await random.position();
println(pos);
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

***

### resetSeed()

> **resetSeed**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Resets the seed to be a random one.

```ts
random.resetSeed();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### setSeed()

> **setSeed**(`seed`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the seed to a value.
This seed is used for all random number generation. Since the random number generator is
deterministic that means that setting it to a particular number will always generate the same
random numbers.

```ts
random.setSeed(42);
```

#### Parameters

##### seed

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### string()

> **string**(`length`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`RandomStringOptions`](RandomStringOptions.md)): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a random string of the given length.

```ts
const token = random.string(16);
```

```ts
const code = random.string(8, { characters: "ABCDEF0123456789" });
```

#### Parameters

##### length

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`RandomStringOptions`](RandomStringOptions.md)

<div class="options-fields">

###### allowLetters?

> `optional` **allowLetters**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Include letters `A-Z` and `a-z` in the default character set.
Ignored when `characters` is specified.

###### Default Value

`true`

***

###### allowNumbers?

> `optional` **allowNumbers**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Include digits `0-9` in the default character set.
Ignored when `characters` is specified.

###### Default Value

`true`

***

###### allowSpecialCharacters?

> `optional` **allowSpecialCharacters**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Include printable ASCII non-alphanumeric characters in the default character set.
Ignored when `characters` is specified.

###### Default Value

`true`

***

###### characters?

> `optional` **characters**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Possible characters to pick from.
Can contain any Unicode grapheme cluster.
When `characters` is specified, `allowNumbers`, `allowLetters` and `allowSpecialCharacters` are ignored.

###### Default Value

```ts
all printable ASCII characters
```

</div>

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
