# Interface: Match

Defined in: [index.d.ts:4010](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4010)

A match returned by a findImage or findImageAll call.

```ts
const source = await Image.load("screenshot.png");
const template = await Image.load("button.png");
const match = await source.findImage(template);
if (match) {
console.log(`Found at ${match.position} with score ${match.score}`);
console.log(`Bounding rect: ${match.rect}`);
}
```

## Properties

### position

> **position**: [`Point`](../classes/Point.md)

Defined in: [index.d.ts:4014](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4014)

the position on the source image where the target image was found

***

### rect

> **rect**: [`Rect`](../classes/Rect.md)

Defined in: [index.d.ts:4018](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4018)

the rectangle on the source image where the target image was found

***

### score

> **score**: `number`

Defined in: [index.d.ts:4022](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4022)

the score for this match, goes from 0 (worst) to 1 (best)

## Methods

### clone()

> **clone**(): `Match`

Defined in: [index.d.ts:4034](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4034)

Clones this Match.

#### Returns

`Match`

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:4026](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4026)

Returns true if a Match equals another.

#### Parameters

##### other

`Match`

#### Returns

`boolean`

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:4030](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4030)

Returns a string representation of this Match.

#### Returns

`string`
