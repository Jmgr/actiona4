# Class: Image

An image that can be loaded, created, manipulated, and saved.

Provides methods for image processing (blur, rotate, resize, color adjustments),
drawing primitives (lines, circles, rectangles, text), and template matching (findImage).

Most mutating methods return `this` for chaining. Each also has an immutable variant
that returns a new `Image` (e.g., `blur()` vs `blurred()`).

```ts
// Create, manipulate, and save
let image = new Image(200, 100);
image.fill(Color.White)
     .drawCircle(100, 50, 30, Color.Red)
     .drawText(10, 10, "Hello", "/path/to/font.ttf", Color.Black);
await image.save("output.png");
```

```ts
// Load, transform, and save
let photo = await Image.load("photo.png");
photo.resize(800, 600, { keepAspectRatio: true })
     .adjustBrightness(10)
     .adjustContrast(5);
await photo.save("photo_edited.png");
```

```ts
// Find an image within another
const screenshot = await Image.load("screenshot.png");
const button = await Image.load("button.png");
const match = await screenshot.find(button, { matchThreshold: 0.9 });
if (match) {
  println(`Button found at ${match.position}`);
}
```

## Constructors

### Constructor

> **new Image**(`width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Creates a new empty image.

Example
```js
let image = new Image(100, 100);
```

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

## Properties

### height

> `readonly` **height**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

***

### rect

> `readonly` **rect**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Rect`](Rect.md)\>

Returns a Rect representing this image.

***

### size

> `readonly` **size**: [`Size`](Size.md)

***

### width

> `readonly` **width**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

## Methods

### adjustBrightness()

> **adjustBrightness**(`value`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Brightens or darkens the pixels of this image.

`value` is added to each RGB channel and clamped to 0–255.
Range: -255 to 255, where 0 = no change, positive = brighter, negative = darker.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`this`

***

### adjustContrast()

> **adjustContrast**(`value`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Adjusts the contrast of this image.

`value` is an arbitrary adjustment where 0 = no change, positive values increase contrast,
and negative values decrease it. At -100 all pixels collapse to 50% gray.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`this`

***

### blur()

> **blur**(`options?`: [`BlurOptions`](../interfaces/BlurOptions.md)): `this`

Blur the image.

#### Parameters

##### options?

[`BlurOptions`](../interfaces/BlurOptions.md)

<div class="options-fields">

###### fast?

> `optional` **fast**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Perform a fast, lower quality blur

###### Default Value

`false`

***

###### sigma?

> `optional` **sigma**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Standard deviation of the (approximated) Gaussian

###### Default Value

`2`

</div>

#### Returns

`this`

***

### blurred()

> **blurred**(`options?`: [`BlurOptions`](../interfaces/BlurOptions.md)): `Image`

Blur the image and returns a new image.

#### Parameters

##### options?

[`BlurOptions`](../interfaces/BlurOptions.md)

<div class="options-fields">

###### fast?

> `optional` **fast**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Perform a fast, lower quality blur

###### Default Value

`false`

***

###### sigma?

> `optional` **sigma**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Standard deviation of the (approximated) Gaussian

###### Default Value

`2`

</div>

#### Returns

`Image`

***

### clone()

> **clone**(): `Image`

Clones this image.

#### Returns

`Image`

***

### copyRegion()

#### Call Signature

> **copyRegion**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): `Image`

Creates a new image from a part of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Image`

#### Call Signature

> **copyRegion**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Creates a new image from a part of this image.

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

`Image`

***

### crop()

#### Call Signature

> **crop**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): `this`

Crops this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`this`

#### Call Signature

> **crop**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Crops this image.

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

`this`

***

### cropped()

#### Call Signature

> **cropped**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): `Image`

Returns a cropped version of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Image`

#### Call Signature

> **cropped**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Returns a cropped version of this image.

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

`Image`

***

### drawCircle()

#### Call Signature

> **drawCircle**(`center`: [`PointLike`](../type-aliases/PointLike.md), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a circle on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawCircle**(`center`: [`PointLike`](../type-aliases/PointLike.md), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a circle on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawCircle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a circle on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawCircle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a circle on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

***

### drawCross()

#### Call Signature

> **drawCross**(`position`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Draw a cross on this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawCross**(`position`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Draw a cross on this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

#### Call Signature

> **drawCross**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Draw a cross on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawCross**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Draw a cross on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

***

### drawEllipse()

#### Call Signature

> **drawEllipse**(`center`: [`PointLike`](../type-aliases/PointLike.md), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw an ellipse on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`center`: [`PointLike`](../type-aliases/PointLike.md), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw an ellipse on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw an ellipse on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw an ellipse on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

***

### drawImage()

#### Call Signature

> **drawImage**(`position`: [`PointLike`](../type-aliases/PointLike.md), `options?`: [`DrawImageOptions`](../interfaces/DrawImageOptions.md)): `this`

Draw another image on this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

<div class="options-fields">

###### sourceRect?

> `optional` **sourceRect**: [`Rect`](Rect.md)

Source rectangle.
[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) means the whole image.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

##### Returns

`this`

#### Call Signature

> **drawImage**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawImageOptions`](../interfaces/DrawImageOptions.md)): `this`

Draw another image on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

<div class="options-fields">

###### sourceRect?

> `optional` **sourceRect**: [`Rect`](Rect.md)

Source rectangle.
[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) means the whole image.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

##### Returns

`this`

***

### drawLine()

#### Call Signature

> **drawLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `end`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `end`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

#### Call Signature

> **drawLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

#### Call Signature

> **drawLine**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `end`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Draw a line on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `end`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Draw a line on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

#### Call Signature

> **drawLine**(`x1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `x2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Draw a line on this image.

##### Parameters

###### x1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### x2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`x1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `x2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Draw a line on this image.

##### Parameters

###### x1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### x2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

***

### drawRectangle()

#### Call Signature

> **drawRectangle**(`rect`: [`RectLike`](../type-aliases/RectLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a rectangle on this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`rect`: [`RectLike`](../type-aliases/RectLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a rectangle on this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a rectangle on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `this`

Draw a rectangle on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`this`

***

### drawText()

#### Call Signature

> **drawText**(`position`: [`PointLike`](../type-aliases/PointLike.md), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `this`

Draw text on this image using the provided font.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`this`

#### Call Signature

> **drawText**(`position`: [`PointLike`](../type-aliases/PointLike.md), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `this`

Draw text on this image using the provided font.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`this`

#### Call Signature

> **drawText**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `this`

Draw text on this image using the provided font.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`this`

#### Call Signature

> **drawText**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `this`

Draw text on this image using the provided font.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`this`

***

### equals()

> **equals**(`other`: `Image`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if this image equals another (same dimensions and pixel data).

#### Parameters

##### other

`Image`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### fill()

#### Call Signature

> **fill**(`color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Fill this image with a color.

##### Parameters

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **fill**(`r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Fill this image with a color.

##### Parameters

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

***

### filled()

#### Call Signature

> **filled**(`color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Returns a copy of this image filled with a color.

##### Parameters

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **filled**(`r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Returns a copy of this image filled with a color.

##### Parameters

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

***

### find()

> <span class="async-badge">async</span> **find**(`image`: `Image`, `options?`: [`FindImageOptions`](../interfaces/FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Finds the best match of an image inside this image.

Returns a `ProgressTask` that can be awaited for the result and iterated
for progress updates. Returns [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if no match is found.

```ts
const match = await source.find(template);
if (match) {
  println(`Found at ${match.position} with score ${match.score}`);
}
```

```ts
// Track progress while searching
const task = source.find(template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

#### Parameters

##### image

`Image`

##### options?

[`FindImageOptions`](../interfaces/FindImageOptions.md)

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

> `optional` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`false`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

***

### findAll()

> <span class="async-badge">async</span> **findAll**(`image`: `Image`, `options?`: [`FindImageOptions`](../interfaces/FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Finds all occurrences of an image inside this image.

Returns a `ProgressTask` that can be awaited for an array of matches.

```ts
const matches = await source.findAll(template, { matchThreshold: 0.85 });
for (const match of matches) {
  println(`Found at ${match.position}`);
}
```

```ts
// Track progress while searching
const task = source.findAll(template);
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

#### Parameters

##### image

`Image`

##### options?

[`FindImageOptions`](../interfaces/FindImageOptions.md)

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

> `optional` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`false`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

***

### findAllOnScreen()

> <span class="async-badge">async</span> **findAllOnScreen**(`searchIn`: [`SearchIn`](SearchIn.md), `options?`: [`FindImageOptions`](../interfaces/FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Finds all matches of this image within the given screen area.

Takes a live screenshot of the specified area and searches for all occurrences.

```ts
const matches = await image.findAllOnScreen(SearchIn.desktop());
for (const match of matches) {
  println(`Found at ${match.position}`);
}
```

```ts
const task = image.findAllOnScreen(SearchIn.rect(0, 0, 1920, 1080));
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const matches = await task;
```

#### Parameters

##### searchIn

[`SearchIn`](SearchIn.md)

##### options?

[`FindImageOptions`](../interfaces/FindImageOptions.md)

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

> `optional` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`false`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### findOnScreen()

> <span class="async-badge">async</span> **findOnScreen**(`searchIn`: [`SearchIn`](SearchIn.md), `options?`: [`FindImageOptions`](../interfaces/FindImageOptions.md)): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Finds the best match of this image within the given screen area.

Takes a live screenshot of the specified area and searches for this image within it.
Returns [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if no match is found.

```ts
const match = await image.findOnScreen(SearchIn.desktop());
if (match) {
  println(`Found at ${match.position} with score ${match.score}`);
}
```

```ts
const display = displays.primary();
const task = image.findOnScreen(SearchIn.display(display));
for await (const progress of task) {
  println(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

#### Parameters

##### searchIn

[`SearchIn`](SearchIn.md)

##### options?

[`FindImageOptions`](../interfaces/FindImageOptions.md)

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

> `optional` **signal**: [`AbortSignal`](../interfaces/AbortSignal.md)

Abort signal to cancel the search.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### useColors?

> `optional` **useColors**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use color matching.

###### Default Value

`false`

***

###### useTransparency?

> `optional` **useTransparency**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Use template transparency.

###### Default Value

`true`

</div>

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### flip()

> **flip**(`flipDirection`: [`FlipDirection`](../enumerations/FlipDirection.md)): `this`

Flip the image.

#### Parameters

##### flipDirection

[`FlipDirection`](../enumerations/FlipDirection.md)

<div class="options-fields">

###### Horizontal

> **Horizontal**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FlipDirection.Horizontal`

***

###### Vertical

> **Vertical**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FlipDirection.Vertical`

</div>

#### Returns

`this`

***

### flipped()

> **flipped**(`flipDirection`: [`FlipDirection`](../enumerations/FlipDirection.md)): `Image`

Flip the image and returns a new image.

#### Parameters

##### flipDirection

[`FlipDirection`](../enumerations/FlipDirection.md)

<div class="options-fields">

###### Horizontal

> **Horizontal**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FlipDirection.Horizontal`

***

###### Vertical

> **Vertical**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`FlipDirection.Vertical`

</div>

#### Returns

`Image`

***

### getPixel()

#### Call Signature

> **getPixel**(`position`: [`PointLike`](../type-aliases/PointLike.md)): [`Color`](Color.md)

Returns the value of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Color`](Color.md)

#### Call Signature

> **getPixel**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Color`](Color.md)

Returns the value of a pixel.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Color`](Color.md)

***

### grayscale()

> **grayscale**(): `this`

Transform this image into a grayscale.

#### Returns

`this`

***

### hueRotate()

> **hueRotate**(`value`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Rotates the hue of each pixel by `value` degrees.

`value` is in degrees and wraps around, so 360 is equivalent to 0.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`this`

***

### invertColors()

> **invertColors**(): `this`

Invert the colors of this image.

#### Returns

`this`

***

### invertedColors()

> **invertedColors**(): `Image`

Invert the colors of this image and returns a new image.

#### Returns

`Image`

***

### resize()

> **resize**(`width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`ResizeOptions`](../interfaces/ResizeOptions.md)): `this`

Resizes this image.

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`ResizeOptions`](../interfaces/ResizeOptions.md)

<div class="options-fields">

###### filter?

> `optional` **filter**: [`ResizeFilter`](../enumerations/ResizeFilter.md)

<div class="options-fields">

###### Cubic

> **Cubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Cubic`

***

###### Gaussian

> **Gaussian**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Gaussian`

***

###### Lanczos3

> **Lanczos3**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Lanczos3`

***

###### Linear

> **Linear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Linear`

***

###### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Nearest`

</div>

What filter to use

###### Default Value

`ResizeFilter.Cubic`

***

###### keepAspectRatio?

> `optional` **keepAspectRatio**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the aspect ratio be kept?

###### Default Value

`false`

</div>

#### Returns

`this`

***

### resized()

> **resized**(`width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`ResizeOptions`](../interfaces/ResizeOptions.md)): `Image`

Returns a resized version of this image.

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`ResizeOptions`](../interfaces/ResizeOptions.md)

<div class="options-fields">

###### filter?

> `optional` **filter**: [`ResizeFilter`](../enumerations/ResizeFilter.md)

<div class="options-fields">

###### Cubic

> **Cubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Cubic`

***

###### Gaussian

> **Gaussian**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Gaussian`

***

###### Lanczos3

> **Lanczos3**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Lanczos3`

***

###### Linear

> **Linear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Linear`

***

###### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`ResizeFilter.Nearest`

</div>

What filter to use

###### Default Value

`ResizeFilter.Cubic`

***

###### keepAspectRatio?

> `optional` **keepAspectRatio**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the aspect ratio be kept?

###### Default Value

`false`

</div>

#### Returns

`Image`

***

### rotate()

> **rotate**(`angle`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`RotationOptions`](../interfaces/RotationOptions.md)): `this`

Rotate the image.

#### Parameters

##### angle

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`RotationOptions`](../interfaces/RotationOptions.md)

<div class="options-fields">

###### center?

> `optional` **center**: [`Point`](Point.md)

Rotation center

###### Default Value

```ts
image center
```

***

###### defaultColor?

> `optional` **defaultColor**: [`Color`](Color.md)

Default color, used if the rotation triggers more pixels to be displayed

###### Default Value

`Color.Black`

***

###### interpolation?

> `optional` **interpolation**: [`Interpolation`](../enumerations/Interpolation.md)

<div class="options-fields">

###### Bicubic

> **Bicubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bicubic`

***

###### Bilinear

> **Bilinear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bilinear`

***

###### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Nearest`

</div>

Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)

###### Default Value

`Interpolation.Bilinear`

</div>

#### Returns

`this`

***

### rotated()

> **rotated**(`angle`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`RotationOptions`](../interfaces/RotationOptions.md)): `Image`

Rotate the image and returns a new image.

#### Parameters

##### angle

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`RotationOptions`](../interfaces/RotationOptions.md)

<div class="options-fields">

###### center?

> `optional` **center**: [`Point`](Point.md)

Rotation center

###### Default Value

```ts
image center
```

***

###### defaultColor?

> `optional` **defaultColor**: [`Color`](Color.md)

Default color, used if the rotation triggers more pixels to be displayed

###### Default Value

`Color.Black`

***

###### interpolation?

> `optional` **interpolation**: [`Interpolation`](../enumerations/Interpolation.md)

<div class="options-fields">

###### Bicubic

> **Bicubic**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bicubic`

***

###### Bilinear

> **Bilinear**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Bilinear`

***

###### Nearest

> **Nearest**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Interpolation.Nearest`

</div>

Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)

###### Default Value

`Interpolation.Bilinear`

</div>

#### Returns

`Image`

***

### save()

> <span class="async-badge">async</span> **save**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Saves this image to a file. The format is inferred from the file extension.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setPixel()

#### Call Signature

> **setPixel**(`position`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Sets the color of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **setPixel**(`position`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Sets the color of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

#### Call Signature

> **setPixel**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `this`

Sets the color of a pixel.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **setPixel**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `this`

Sets the color of a pixel.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`this`

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this image.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### withAdjustedBrightness()

> **withAdjustedBrightness**(`value`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Returns a brightened or darkened copy of this image.

`value` is added to each RGB channel and clamped to 0–255.
Range: -255 to 255, where 0 = no change, positive = brighter, negative = darker.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

***

### withAdjustedContrast()

> **withAdjustedContrast**(`value`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Returns a contrast-adjusted copy of this image (0 = no change, positive = more contrast, -100 = all pixels become 50% gray).

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

***

### withCircle()

#### Call Signature

> **withCircle**(`center`: [`PointLike`](../type-aliases/PointLike.md), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a circle on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withCircle**(`center`: [`PointLike`](../type-aliases/PointLike.md), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a circle on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withCircle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a circle on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withCircle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `radius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a circle on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### radius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

***

### withCross()

#### Call Signature

> **withCross**(`position`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Draw a cross on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withCross**(`position`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Draw a cross on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

#### Call Signature

> **withCross**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Draw a cross on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withCross**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Draw a cross on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

***

### withEllipse()

#### Call Signature

> **withEllipse**(`center`: [`PointLike`](../type-aliases/PointLike.md), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw an ellipse on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`center`: [`PointLike`](../type-aliases/PointLike.md), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw an ellipse on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw an ellipse on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `widthRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `heightRadius`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw an ellipse on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### widthRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### heightRadius

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

***

### withGrayscale()

> **withGrayscale**(): `Image`

Returns a grayscale version of this image.

#### Returns

`Image`

***

### withHueRotation()

> **withHueRotation**(`value`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Returns a hue-rotated copy of this image.

`value` is in degrees and wraps around, so 360 is equivalent to 0.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

***

### withImage()

#### Call Signature

> **withImage**(`position`: [`PointLike`](../type-aliases/PointLike.md), `image`: `Image`, `options?`: [`DrawImageOptions`](../interfaces/DrawImageOptions.md)): `Image`

Draw another image on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### image

`Image`

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

<div class="options-fields">

###### sourceRect?

> `optional` **sourceRect**: [`Rect`](Rect.md)

Source rectangle.
[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) means the whole image.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

##### Returns

`Image`

#### Call Signature

> **withImage**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `image`: `Image`, `options?`: [`DrawImageOptions`](../interfaces/DrawImageOptions.md)): `Image`

Draw another image on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### image

`Image`

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

<div class="options-fields">

###### sourceRect?

> `optional` **sourceRect**: [`Rect`](Rect.md)

Source rectangle.
[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) means the whole image.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

##### Returns

`Image`

***

### withLine()

#### Call Signature

> **withLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `end`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `end`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

#### Call Signature

> **withLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`start`: [`PointLike`](../type-aliases/PointLike.md), `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

#### Call Signature

> **withLine**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `end`: [`PointLike`](../type-aliases/PointLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `end`: [`PointLike`](../type-aliases/PointLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

#### Call Signature

> **withLine**(`x1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `x2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### x1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### x2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`x1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `x2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Image`

Draw a line on a copy of this image.

##### Parameters

###### x1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### x2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Image`

***

### withRectangle()

#### Call Signature

> **withRectangle**(`rect`: [`RectLike`](../type-aliases/RectLike.md), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a rectangle on a copy of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`rect`: [`RectLike`](../type-aliases/RectLike.md), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a rectangle on a copy of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a rectangle on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawingOptions`](../interfaces/DrawingOptions.md)): `Image`

Draw a rectangle on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

<div class="options-fields">

###### hollow?

> `optional` **hollow**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Draw a hollow shape instead of a filled one

###### Default Value

`false`

</div>

##### Returns

`Image`

***

### withText()

#### Call Signature

> **withText**(`position`: [`PointLike`](../type-aliases/PointLike.md), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `Image`

Draw text on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`Image`

#### Call Signature

> **withText**(`position`: [`PointLike`](../type-aliases/PointLike.md), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `Image`

Draw text on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`Image`

#### Call Signature

> **withText**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `color`: [`ColorLike`](../type-aliases/ColorLike.md), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `Image`

Draw text on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`Image`

#### Call Signature

> **withText**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `text`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `fontPath`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DrawTextOptions`](../interfaces/DrawTextOptions.md)): `Image`

Draw text on a copy of this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### text

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### fontPath

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

###### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

<div class="options-fields">

###### fontSize?

> `optional` **fontSize**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Font size in pixels.

###### Default Value

`16`

***

###### horizontalAlign?

> `optional` **horizontalAlign**: [`TextHorizontalAlign`](../enumerations/TextHorizontalAlign.md)

<div class="options-fields">

###### Center

> **Center**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Center`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Left`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextHorizontalAlign.Right`

</div>

Horizontal alignment relative to the provided position.

###### Default Value

`TextHorizontalAlign.Left`

***

###### lineSpacing?

> `optional` **lineSpacing**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Multiplier applied to the default line height when rendering multi-line text.

###### Default Value

`1`

***

###### verticalAlign?

> `optional` **verticalAlign**: [`TextVerticalAlign`](../enumerations/TextVerticalAlign.md)

<div class="options-fields">

###### Bottom

> **Bottom**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Bottom`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Middle`

***

###### Top

> **Top**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`TextVerticalAlign.Top`

</div>

Vertical alignment relative to the provided position.

###### Default Value

`TextVerticalAlign.Top`

</div>

##### Returns

`Image`

***

### fromBytes()

> `static` **fromBytes**(`bytes`: [`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)): `Image`

Creates a new image from raw encoded bytes (PNG, JPEG, etc.).

```ts
const bytes = await file.readAll();
const image = Image.fromBytes(bytes);
```

#### Parameters

##### bytes

[`Uint8Array`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array)

#### Returns

`Image`

***

### load()

> <span class="async-badge">async</span> `static` **load**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Image`\>

Loads an image from a file. The format is guessed from the file contents.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Image`\>
