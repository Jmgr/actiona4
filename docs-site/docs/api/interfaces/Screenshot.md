# Interface: Screenshot

Screenshot capture and image search.

Provides methods to capture the entire desktop, a specific display, a screen
region, or a single pixel, as well as finding images on screen.

```ts
const image = await screenshot.captureDesktop();
println(image.size());
```

```ts
const image = await screenshot.captureDisplay(Display.primary());
println(image.size());
```

```ts
const pixel = await screenshot.capturePixel(100, 100);
println(pixel);
```

## Methods

### captureDesktop()

> <span class="async-badge">async</span> **captureDesktop**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the entire desktop.

```ts
const image = await screenshot.captureDesktop();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### captureDisplay()

> <span class="async-badge">async</span> **captureDisplay**(`display`: [`Display`](../classes/Display.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the display identified by the given selector.

```ts
const image = await screenshot.captureDisplay(Display.primary());
const image = await screenshot.captureDisplay(Display.fromId(474));
const image = await screenshot.captureDisplay(Display.fromName(/HDMI-.*/));
```

#### Parameters

##### display

[`Display`](../classes/Display.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### capturePixel()

#### Call Signature

> <span class="async-badge">async</span> **capturePixel**(`position`: [`PointLike`](../type-aliases/PointLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

Captures the color of a single pixel on screen.

```ts
const color = await screenshot.capturePixel(100, 200);
println(color);
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
println(color);
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

### captureWindow()

> <span class="async-badge">async</span> **captureWindow**(`handle`: [`WindowHandle`](WindowHandle.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the bounding rectangle of the given window.

```ts
const win = windows.activeWindow();
const image = await screenshot.captureWindow(win);
```

#### Parameters

##### handle

[`WindowHandle`](WindowHandle.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

***

### findImage()

> <span class="async-badge">async</span> **findImage**(`searchIn`: [`SearchIn`](../classes/SearchIn.md), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](FindImageProgress.md)\>

Finds the best match of an image within the given search area.

```ts
const match = await screenshot.findImage(SearchIn.desktop(), template);
```

```ts
const task = screenshot.findImage(SearchIn.display(Display.primary()), template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

#### Parameters

##### searchIn

[`SearchIn`](../classes/SearchIn.md)

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

### findImageAll()

> <span class="async-badge">async</span> **findImageAll**(`searchIn`: [`SearchIn`](../classes/SearchIn.md), `image`: [`Image`](../classes/Image.md), `options?`: [`FindImageOptions`](FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](Match.md)[], [`FindImageProgress`](FindImageProgress.md)\>

Finds all matches of an image within the given search area.

```ts
const matches = await screenshot.findImageAll(SearchIn.desktop(), image);
```

```ts
const task = screenshot.findImageAll(SearchIn.rect(0, 0, 1920, 1080), image);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

#### Parameters

##### searchIn

[`SearchIn`](../classes/SearchIn.md)

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

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
