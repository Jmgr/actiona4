# Interface: Screenshot

Defined in: [index.d.ts:5354](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5354)

Screenshot capture and image search.

Provides methods to capture screen regions, displays, and individual pixels,
as well as finding images on screen.

```ts
const image = await screenshot.captureDisplay(0);
console.log(image.size().toString());
```

```ts
const pixel = await screenshot.capturePixel(100, 100);
console.log(pixel.toString());
```

## Methods

### captureDisplay()

> **captureDisplay**(`displayId`): `Promise`\<[`Image`](../classes/Image.md)\>

Defined in: [index.d.ts:5378](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5378)

Captures a screenshot of an entire display.

```ts
const image = await screenshot.captureDisplay(0);
```

#### Parameters

##### displayId

`number`

#### Returns

`Promise`\<[`Image`](../classes/Image.md)\>

***

### capturePixel()

#### Call Signature

> **capturePixel**(`position`): `Promise`\<[`Color`](../classes/Color.md)\>

Defined in: [index.d.ts:5387](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5387)

Captures the color of a single pixel on screen.

```ts
const color = await screenshot.capturePixel(100, 200);
console.log(color.toString());
```

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

`Promise`\<[`Color`](../classes/Color.md)\>

#### Call Signature

> **capturePixel**(`x`, `y`): `Promise`\<[`Color`](../classes/Color.md)\>

Defined in: [index.d.ts:5396](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5396)

Captures the color of a single pixel on screen.

```ts
const color = await screenshot.capturePixel(100, 200);
console.log(color.toString());
```

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`Promise`\<[`Color`](../classes/Color.md)\>

***

### captureRect()

#### Call Signature

> **captureRect**(`rect`): `Promise`\<[`Image`](../classes/Image.md)\>

Defined in: [index.d.ts:5362](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5362)

Captures a screenshot of a screen rectangle.

```ts
const image = await screenshot.captureRect(0, 0, 1920, 1080);
```

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Promise`\<[`Image`](../classes/Image.md)\>

#### Call Signature

> **captureRect**(`x`, `y`, `width`, `height`): `Promise`\<[`Image`](../classes/Image.md)\>

Defined in: [index.d.ts:5370](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5370)

Captures a screenshot of a screen rectangle.

```ts
const image = await screenshot.captureRect(0, 0, 1920, 1080);
```

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

##### Returns

`Promise`\<[`Image`](../classes/Image.md)\>

***

### findImageOnDisplay()

> **findImageOnDisplay**(`displayId`, `image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| `undefined`, [`FindImageProgress`](FindImageProgress.md)\>

Defined in: [index.d.ts:5476](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5476)

Finds the best match of an image on a display.

```ts
const match = await screenshot.findImageOnDisplay(0, template);
```

```ts
const task = screenshot.findImageOnDisplay(0, template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

#### Parameters

##### displayId

`number`

##### image

[`Image`](../classes/Image.md)

##### options?

[`FindImageOptions`](FindImageOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| `undefined`, [`FindImageProgress`](FindImageProgress.md)\>

***

### findImageOnDisplayAll()

> **findImageOnDisplayAll**(`displayId`, `image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Defined in: [index.d.ts:5492](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5492)

Finds all occurrences of an image on a display.

```ts
const matches = await screenshot.findImageOnDisplayAll(0, template);
```

```ts
const task = screenshot.findImageOnDisplayAll(0, template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

#### Parameters

##### displayId

`number`

##### image

[`Image`](../classes/Image.md)

##### options?

[`FindImageOptions`](FindImageOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

***

### findImageOnRect()

#### Call Signature

> **findImageOnRect**(`rect`, `image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| `undefined`, [`FindImageProgress`](FindImageProgress.md)\>

Defined in: [index.d.ts:5412](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5412)

Finds the best match of an image on a screen rectangle.

```ts
const match = await screenshot.findImageOnRect(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRect(0, 0, 1920, 1080, template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### image

[`Image`](../classes/Image.md)

###### options?

[`FindImageOptions`](FindImageOptions.md)

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| `undefined`, [`FindImageProgress`](FindImageProgress.md)\>

#### Call Signature

> **findImageOnRect**(`x`, `y`, `width`, `height`, `image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| `undefined`, [`FindImageProgress`](FindImageProgress.md)\>

Defined in: [index.d.ts:5428](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5428)

Finds the best match of an image on a screen rectangle.

```ts
const match = await screenshot.findImageOnRect(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRect(0, 0, 1920, 1080, template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

###### image

[`Image`](../classes/Image.md)

###### options?

[`FindImageOptions`](FindImageOptions.md)

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| `undefined`, [`FindImageProgress`](FindImageProgress.md)\>

***

### findImageOnRectAll()

#### Call Signature

> **findImageOnRectAll**(`rect`, `image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Defined in: [index.d.ts:5444](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5444)

Finds all occurrences of an image on a screen rectangle.

```ts
const matches = await screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### image

[`Image`](../classes/Image.md)

###### options?

[`FindImageOptions`](FindImageOptions.md)

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

#### Call Signature

> **findImageOnRectAll**(`x`, `y`, `width`, `height`, `image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Defined in: [index.d.ts:5460](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5460)

Finds all occurrences of an image on a screen rectangle.

```ts
const matches = await screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

###### image

[`Image`](../classes/Image.md)

###### options?

[`FindImageOptions`](FindImageOptions.md)

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>
