# Interface: Match

A match returned by a find or findAll call.

```ts
const source = await Image.load("screenshot.png");
const template = await Image.load("button.png");
const match = await source.find(template);
if (match) {
  println(`Found at ${match.position} with score ${match.score}`);
  println(`Bounding rect: ${match.rect}`);
}
```

## Properties

### position

> **position**: [`Point`](../classes/Point.md)

the position on the source image where the target image was found

***

### rect

> **rect**: [`Rect`](../classes/Rect.md)

the rectangle on the source image where the target image was found

***

### score

> **score**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

the score for this match, goes from 0 (worst) to 1 (best)

## Methods

### equals()

> **equals**(`other`: `Match`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if a Match equals another.

#### Parameters

##### other

`Match`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this match.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### clone()

> **clone**(): `Match`

Clones this Match.

#### Returns

`Match`
