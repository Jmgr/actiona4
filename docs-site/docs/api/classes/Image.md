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
const screen = await Image.load("screenshot.png");
const button = await Image.load("button.png");
const match = await screen.findImage(button, { matchThreshold: 0.9 });
if (match) {
  println(`Button found at ${match.position}`);
}
```

## Constructors

### Constructor

> **new Image**(`width`, `height`): `Image`

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

> **adjustBrightness**(`value`): `this`

Brightens or darkens the pixels of this image.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`this`

***

### adjustContrast()

> **adjustContrast**(`value`): `this`

Adjusts the contrast of this image.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`this`

***

### blur()

> **blur**(`options?`): `this`

Blur the image.

#### Parameters

##### options?

[`BlurOptions`](../interfaces/BlurOptions.md)

#### Returns

`this`

***

### blurred()

> **blurred**(`options?`): `Image`

Blur the image and returns a new image.

#### Parameters

##### options?

[`BlurOptions`](../interfaces/BlurOptions.md)

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

> **copyRegion**(`rect`): `Image`

Creates a new image from a part of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Image`

#### Call Signature

> **copyRegion**(`x`, `y`, `width`, `height`): `Image`

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

> **crop**(`rect`): `this`

Crops this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`this`

#### Call Signature

> **crop**(`x`, `y`, `width`, `height`): `this`

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

> **cropped**(`rect`): `Image`

Returns a cropped version of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Image`

#### Call Signature

> **cropped**(`x`, `y`, `width`, `height`): `Image`

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

> **drawCircle**(`center`, `radius`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawCircle**(`center`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawCircle**(`x`, `y`, `radius`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawCircle**(`x`, `y`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

***

### drawCross()

#### Call Signature

> **drawCross**(`position`, `color`): `this`

Draw a cross on this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawCross**(`position`, `r`, `g`, `b`, `a?`): `this`

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

> **drawCross**(`x`, `y`, `color`): `this`

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

> **drawCross**(`x`, `y`, `r`, `g`, `b`, `a?`): `this`

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

> **drawEllipse**(`center`, `widthRadius`, `heightRadius`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`center`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

***

### drawImage()

#### Call Signature

> **drawImage**(`position`, `image`, `options?`): `this`

Draw another image on this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### image

`Image`

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

##### Returns

`this`

#### Call Signature

> **drawImage**(`x`, `y`, `image`, `options?`): `this`

Draw another image on this image.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### image

`Image`

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

##### Returns

`this`

***

### drawLine()

#### Call Signature

> **drawLine**(`start`, `end`, `color`): `this`

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

> **drawLine**(`start`, `end`, `r`, `g`, `b`, `a?`): `this`

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

> **drawLine**(`start`, `x`, `y`, `color`): `this`

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

> **drawLine**(`start`, `x`, `y`, `r`, `g`, `b`, `a?`): `this`

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

> **drawLine**(`x`, `y`, `end`, `color`): `this`

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

> **drawLine**(`x`, `y`, `end`, `r`, `g`, `b`, `a?`): `this`

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

> **drawLine**(`x1`, `y1`, `x2`, `y2`, `color`): `this`

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

> **drawLine**(`x1`, `y1`, `x2`, `y2`, `r`, `g`, `b`, `a?`): `this`

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

> **drawRectangle**(`rect`, `color`, `options?`): `this`

Draw a rectangle on this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`rect`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`x`, `y`, `width`, `height`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`x`, `y`, `width`, `height`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

***

### drawText()

#### Call Signature

> **drawText**(`position`, `text`, `fontPath`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawText**(`position`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawText**(`x`, `y`, `text`, `fontPath`, `color`, `options?`): `this`

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

##### Returns

`this`

#### Call Signature

> **drawText**(`x`, `y`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `this`

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

##### Returns

`this`

***

### equals()

> **equals**(`other`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns true if this image equals another (same dimensions and pixel data).

#### Parameters

##### other

`Image`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### fill()

#### Call Signature

> **fill**(`color`): `this`

Fill this image with a color.

##### Parameters

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **fill**(`r`, `g`, `b`, `a?`): `this`

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

> **filled**(`color`): `Image`

Returns a copy of this image filled with a color.

##### Parameters

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **filled**(`r`, `g`, `b`, `a?`): `Image`

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

### findImage()

> **findImage**(`image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Finds the best match of an image inside this image.

Returns a `ProgressTask` that can be awaited for the result and iterated
for progress updates. Returns [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if no match is found.

```ts
const match = await source.findImage(template);
if (match) {
  println(`Found at ${match.position} with score ${match.score}`);
}
```

```ts
// Track progress while searching
const task = source.findImage(template);
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

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined), [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

***

### findImageAll()

> **findImageAll**(`image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Finds all occurrences of an image inside this image.

Returns a `ProgressTask` that can be awaited for an array of matches.

```ts
const matches = await source.findImageAll(template, { matchThreshold: 0.85 });
for (const match of matches) {
  println(`Found at ${match.position}`);
}
```

```ts
// Track progress while searching
const task = source.findImageAll(template);
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

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

***

### flip()

> **flip**(`flipDirection`): `this`

Flip the image.

#### Parameters

##### flipDirection

[`FlipDirection`](../enumerations/FlipDirection.md)

#### Returns

`this`

***

### flipped()

> **flipped**(`flipDirection`): `Image`

Flip the image and returns a new image.

#### Parameters

##### flipDirection

[`FlipDirection`](../enumerations/FlipDirection.md)

#### Returns

`Image`

***

### getPixel()

#### Call Signature

> **getPixel**(`position`): [`Color`](Color.md)

Returns the value of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Color`](Color.md)

#### Call Signature

> **getPixel**(`x`, `y`): [`Color`](Color.md)

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

> **hueRotate**(`value`): `this`

Hue rotate the image.

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

> **resize**(`width`, `height`, `options?`): `this`

Resizes this image.

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`ResizeOptions`](../interfaces/ResizeOptions.md)

#### Returns

`this`

***

### resized()

> **resized**(`width`, `height`, `options?`): `Image`

Returns a resized version of this image.

#### Parameters

##### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`ResizeOptions`](../interfaces/ResizeOptions.md)

#### Returns

`Image`

***

### rotate()

> **rotate**(`angle`, `options?`): `this`

Rotate the image.

#### Parameters

##### angle

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`RotationOptions`](../interfaces/RotationOptions.md)

#### Returns

`this`

***

### rotated()

> **rotated**(`angle`, `options?`): `Image`

Rotate the image and returns a new image.

#### Parameters

##### angle

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### options?

[`RotationOptions`](../interfaces/RotationOptions.md)

#### Returns

`Image`

***

### save()

> **save**(`path`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Saves this image to a file. The format is inferred from the file extension.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setPixel()

#### Call Signature

> **setPixel**(`position`, `color`): `this`

Sets the color of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **setPixel**(`position`, `r`, `g`, `b`, `a?`): `this`

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

> **setPixel**(`x`, `y`, `color`): `this`

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

> **setPixel**(`x`, `y`, `r`, `g`, `b`, `a?`): `this`

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

> **withAdjustedBrightness**(`value`): `Image`

Returns a brightened or darkened version of this image.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

***

### withAdjustedContrast()

> **withAdjustedContrast**(`value`): `Image`

Returns a new image with an adjusted contrast.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

***

### withCircle()

#### Call Signature

> **withCircle**(`center`, `radius`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withCircle**(`center`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withCircle**(`x`, `y`, `radius`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withCircle**(`x`, `y`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

***

### withCross()

#### Call Signature

> **withCross**(`position`, `color`): `Image`

Draw a cross on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withCross**(`position`, `r`, `g`, `b`, `a?`): `Image`

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

> **withCross**(`x`, `y`, `color`): `Image`

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

> **withCross**(`x`, `y`, `r`, `g`, `b`, `a?`): `Image`

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

> **withEllipse**(`center`, `widthRadius`, `heightRadius`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`center`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

> **withHueRotation**(`value`): `Image`

Hue rotate the image and returns a new image.

#### Parameters

##### value

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Image`

***

### withImage()

#### Call Signature

> **withImage**(`position`, `image`, `options?`): `Image`

Draw another image on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### image

`Image`

###### options?

[`DrawImageOptions`](../interfaces/DrawImageOptions.md)

##### Returns

`Image`

#### Call Signature

> **withImage**(`x`, `y`, `image`, `options?`): `Image`

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

##### Returns

`Image`

***

### withLine()

#### Call Signature

> **withLine**(`start`, `end`, `color`): `Image`

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

> **withLine**(`start`, `end`, `r`, `g`, `b`, `a?`): `Image`

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

> **withLine**(`start`, `x`, `y`, `color`): `Image`

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

> **withLine**(`start`, `x`, `y`, `r`, `g`, `b`, `a?`): `Image`

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

> **withLine**(`x`, `y`, `end`, `color`): `Image`

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

> **withLine**(`x`, `y`, `end`, `r`, `g`, `b`, `a?`): `Image`

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

> **withLine**(`x1`, `y1`, `x2`, `y2`, `color`): `Image`

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

> **withLine**(`x1`, `y1`, `x2`, `y2`, `r`, `g`, `b`, `a?`): `Image`

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

> **withRectangle**(`rect`, `color`, `options?`): `Image`

Draw a rectangle on a copy of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`rect`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`x`, `y`, `width`, `height`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`x`, `y`, `width`, `height`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

***

### withText()

#### Call Signature

> **withText**(`position`, `text`, `fontPath`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withText**(`position`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withText**(`x`, `y`, `text`, `fontPath`, `color`, `options?`): `Image`

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

##### Returns

`Image`

#### Call Signature

> **withText**(`x`, `y`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `Image`

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

##### Returns

`Image`

***

### fromBytes()

> `static` **fromBytes**(`bytes`): `Image`

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

> `static` **load**(`path`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Image`\>

Loads an image from a file. The format is guessed from the file contents.

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Image`\>
