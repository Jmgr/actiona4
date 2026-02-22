# Interface: Screenshot

Screenshot capture and image search.

Provides methods to capture screen regions, displays, and individual pixels,
as well as finding images on screen.

```ts
const image = await screenshot.captureDisplay(0);
println(image.size().toString());
```

```ts
const pixel = await screenshot.capturePixel(100, 100);
println(pixel.toString());
```

## Methods

### captureDisplay()

> <span class="async-badge">async</span> **captureDisplay**(`displayId`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of an entire display.

```ts
const image = await screenshot.captureDisplay(0);
```

#### Parameters

##### displayId

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### capturePixel()

#### Call Signature

> <span class="async-badge">async</span> **capturePixel**(`position`: [`PointLike`](../type-aliases/PointLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

Captures the color of a single pixel on screen.

```ts
const color = await screenshot.capturePixel(100, 200);
println(color.toString());
```

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

#### Call Signature

> <span class="async-badge">async</span> **capturePixel**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

Captures the color of a single pixel on screen.

```ts
const color = await screenshot.capturePixel(100, 200);
println(color.toString());
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

***

### captureRect()

#### Call Signature

> <span class="async-badge">async</span> **captureRect**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of a screen rectangle.

```ts
const image = await screenshot.captureRect(0, 0, 1920, 1080);
```

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

#### Call Signature

> <span class="async-badge">async</span> **captureRect**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of a screen rectangle.

```ts
const image = await screenshot.captureRect(0, 0, 1920, 1080);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### findImageOnDisplay()

> <span class="async-badge">async</span> **findImageOnDisplay**(`displayId`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

Finds the best match of an image on a display.

```ts
const match = await screenshot.findImageOnDisplay(0, template);
```

```ts
const task = screenshot.findImageOnDisplay(0, template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

#### Parameters

##### displayId

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### image

[`Image`](../classes/Image.md)

##### options?

[`FindImageOptions`](FindImageOptions.md)

<div class="options-fields">

###### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

###### Default Value

`0`

***

###### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

###### Default Value

`0.8`

***

###### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

###### Default Value

`10`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`true`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

***

### findImageOnDisplayAll()

> <span class="async-badge">async</span> **findImageOnDisplayAll**(`displayId`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Finds all occurrences of an image on a display.

```ts
const matches = await screenshot.findImageOnDisplayAll(0, template);
```

```ts
const task = screenshot.findImageOnDisplayAll(0, template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

#### Parameters

##### displayId

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### image

[`Image`](../classes/Image.md)

##### options?

[`FindImageOptions`](FindImageOptions.md)

<div class="options-fields">

###### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

###### Default Value

`0`

***

###### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

###### Default Value

`0.8`

***

###### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

###### Default Value

`10`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`true`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

***

### findImageOnRect()

#### Call Signature

> <span class="async-badge">async</span> **findImageOnRect**(`rect`: [`RectLike`](../type-aliases/RectLike.md), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

Finds the best match of an image on a screen rectangle.

```ts
const match = await screenshot.findImageOnRect(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRect(0, 0, 1920, 1080, template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
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

<div class="options-fields">

###### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

###### Default Value

`0`

***

###### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

###### Default Value

`0.8`

***

###### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

###### Default Value

`10`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`true`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

#### Call Signature

> <span class="async-badge">async</span> **findImageOnRect**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

Finds the best match of an image on a screen rectangle.

```ts
const match = await screenshot.findImageOnRect(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRect(0, 0, 1920, 1080, template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### image

[`Image`](../classes/Image.md)

###### options?

[`FindImageOptions`](FindImageOptions.md)

<div class="options-fields">

###### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

###### Default Value

`0`

***

###### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

###### Default Value

`0.8`

***

###### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

###### Default Value

`10`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`true`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

***

### findImageOnRectAll()

#### Call Signature

> <span class="async-badge">async</span> **findImageOnRectAll**(`rect`: [`RectLike`](../type-aliases/RectLike.md), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Finds all occurrences of an image on a screen rectangle.

```ts
const matches = await screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
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

<div class="options-fields">

###### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

###### Default Value

`0`

***

###### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

###### Default Value

`0.8`

***

###### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

###### Default Value

`10`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`true`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

#### Call Signature

> <span class="async-badge">async</span> **findImageOnRectAll**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Finds all occurrences of an image on a screen rectangle.

```ts
const matches = await screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
```

```ts
const task = screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### image

[`Image`](../classes/Image.md)

###### options?

[`FindImageOptions`](FindImageOptions.md)

<div class="options-fields">

###### downscale?

> `optional` **downscale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How many times should the source image and the template be downscaled?

###### Default Value

`0`

***

###### matchThreshold?

> `optional` **matchThreshold**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Matching threshold.
Values are between 0 (worst) to 1 (best).

###### Default Value

`0.8`

***

###### nonMaximumSuppressionRadius?

> `optional` **nonMaximumSuppressionRadius**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Radius to consider proximity (in pixels).

###### Default Value

`10`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`true`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

##### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
