# Class: Image

Defined in: [index.d.ts:4103](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4103)

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
console.log(`Button found at ${match.position}`);
}
```

## Constructors

### Constructor

> **new Image**(`width`, `height`): `Image`

Defined in: [index.d.ts:4118](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4118)

Creates a new empty image.

Example
```js
let image = new Image(100, 100);
```

#### Parameters

##### width

`number`

##### height

`number`

#### Returns

`Image`

## Properties

### height

> `readonly` **height**: `number`

Defined in: [index.d.ts:4105](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4105)

***

### rect

> `readonly` **rect**: `Readonly`\<[`Rect`](Rect.md)\>

Defined in: [index.d.ts:4109](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4109)

Returns a Rect representing this image.

***

### width

> `readonly` **width**: `number`

Defined in: [index.d.ts:4104](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4104)

## Methods

### adjustBrightness()

> **adjustBrightness**(`value`): `this`

Defined in: [index.d.ts:4223](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4223)

Brightens or darkens the pixels of this image.

#### Parameters

##### value

`number`

#### Returns

`this`

***

### adjustContrast()

> **adjustContrast**(`value`): `this`

Defined in: [index.d.ts:4231](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4231)

Adjusts the contrast of this image.

#### Parameters

##### value

`number`

#### Returns

`this`

***

### blur()

> **blur**(`options?`): `this`

Defined in: [index.d.ts:4159](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4159)

Blur the image.

#### Parameters

##### options?

[`BlurOptions`](../interfaces/BlurOptions.md)

#### Returns

`this`

***

### blurred()

> **blurred**(`options?`): `Image`

Defined in: [index.d.ts:4163](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4163)

Blur the image and returns a new image.

#### Parameters

##### options?

[`BlurOptions`](../interfaces/BlurOptions.md)

#### Returns

`Image`

***

### clone()

> **clone**(): `Image`

Defined in: [index.d.ts:4147](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4147)

Clones this image.

#### Returns

`Image`

***

### copyRegion()

#### Call Signature

> **copyRegion**(`rect`): `Image`

Defined in: [index.d.ts:4279](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4279)

Creates a new image from a part of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Image`

#### Call Signature

> **copyRegion**(`x`, `y`, `width`, `height`): `Image`

Defined in: [index.d.ts:4283](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4283)

Creates a new image from a part of this image.

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

`Image`

***

### crop()

#### Call Signature

> **crop**(`rect`): `this`

Defined in: [index.d.ts:4199](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4199)

Crops this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`this`

#### Call Signature

> **crop**(`x`, `y`, `width`, `height`): `this`

Defined in: [index.d.ts:4203](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4203)

Crops this image.

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

`this`

***

### cropped()

#### Call Signature

> **cropped**(`rect`): `Image`

Defined in: [index.d.ts:4207](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4207)

Returns a cropped version of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`Image`

#### Call Signature

> **cropped**(`x`, `y`, `width`, `height`): `Image`

Defined in: [index.d.ts:4211](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4211)

Returns a cropped version of this image.

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

`Image`

***

### drawCircle()

#### Call Signature

> **drawCircle**(`center`, `radius`, `color`, `options?`): `this`

Defined in: [index.d.ts:4383](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4383)

Draw a circle on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawCircle**(`center`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4387](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4387)

Draw a circle on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawCircle**(`x`, `y`, `radius`, `color`, `options?`): `this`

Defined in: [index.d.ts:4391](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4391)

Draw a circle on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### radius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawCircle**(`x`, `y`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4395](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4395)

Draw a circle on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### radius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

***

### drawCross()

#### Call Signature

> **drawCross**(`position`, `color`): `this`

Defined in: [index.d.ts:4287](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4287)

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

Defined in: [index.d.ts:4291](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4291)

Draw a cross on this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

#### Call Signature

> **drawCross**(`x`, `y`, `color`): `this`

Defined in: [index.d.ts:4295](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4295)

Draw a cross on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawCross**(`x`, `y`, `r`, `g`, `b`, `a?`): `this`

Defined in: [index.d.ts:4299](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4299)

Draw a cross on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

***

### drawEllipse()

#### Call Signature

> **drawEllipse**(`center`, `widthRadius`, `heightRadius`, `color`, `options?`): `this`

Defined in: [index.d.ts:4415](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4415)

Draw an ellipse on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

`number`

###### heightRadius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`center`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4419](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4419)

Draw an ellipse on this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

`number`

###### heightRadius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `color`, `options?`): `this`

Defined in: [index.d.ts:4423](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4423)

Draw an ellipse on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### widthRadius

`number`

###### heightRadius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4427](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4427)

Draw an ellipse on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### widthRadius

`number`

###### heightRadius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

***

### drawImage()

#### Call Signature

> **drawImage**(`position`, `image`, `options?`): `this`

Defined in: [index.d.ts:4511](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4511)

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

Defined in: [index.d.ts:4515](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4515)

Draw another image on this image.

##### Parameters

###### x

`number`

###### y

`number`

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

Defined in: [index.d.ts:4319](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4319)

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

Defined in: [index.d.ts:4323](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4323)

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

#### Call Signature

> **drawLine**(`start`, `x`, `y`, `color`): `this`

Defined in: [index.d.ts:4327](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4327)

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

`number`

###### y

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`start`, `x`, `y`, `r`, `g`, `b`, `a?`): `this`

Defined in: [index.d.ts:4331](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4331)

Draw a line on this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

`number`

###### y

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

#### Call Signature

> **drawLine**(`x`, `y`, `end`, `color`): `this`

Defined in: [index.d.ts:4335](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4335)

Draw a line on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`x`, `y`, `end`, `r`, `g`, `b`, `a?`): `this`

Defined in: [index.d.ts:4339](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4339)

Draw a line on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

#### Call Signature

> **drawLine**(`x1`, `y1`, `x2`, `y2`, `color`): `this`

Defined in: [index.d.ts:4343](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4343)

Draw a line on this image.

##### Parameters

###### x1

`number`

###### y1

`number`

###### x2

`number`

###### y2

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **drawLine**(`x1`, `y1`, `x2`, `y2`, `r`, `g`, `b`, `a?`): `this`

Defined in: [index.d.ts:4347](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4347)

Draw a line on this image.

##### Parameters

###### x1

`number`

###### y1

`number`

###### x2

`number`

###### y2

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

***

### drawRectangle()

#### Call Signature

> **drawRectangle**(`rect`, `color`, `options?`): `this`

Defined in: [index.d.ts:4447](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4447)

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

Defined in: [index.d.ts:4451](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4451)

Draw a rectangle on this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`x`, `y`, `width`, `height`, `color`, `options?`): `this`

Defined in: [index.d.ts:4455](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4455)

Draw a rectangle on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

#### Call Signature

> **drawRectangle**(`x`, `y`, `width`, `height`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4459](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4459)

Draw a rectangle on this image.

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`this`

***

### drawText()

#### Call Signature

> **drawText**(`position`, `text`, `fontPath`, `color`, `options?`): `this`

Defined in: [index.d.ts:4479](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4479)

Draw text on this image using the provided font.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

`string`

###### fontPath

`string`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`this`

#### Call Signature

> **drawText**(`position`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4483](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4483)

Draw text on this image using the provided font.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

`string`

###### fontPath

`string`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`this`

#### Call Signature

> **drawText**(`x`, `y`, `text`, `fontPath`, `color`, `options?`): `this`

Defined in: [index.d.ts:4487](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4487)

Draw text on this image using the provided font.

##### Parameters

###### x

`number`

###### y

`number`

###### text

`string`

###### fontPath

`string`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`this`

#### Call Signature

> **drawText**(`x`, `y`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `this`

Defined in: [index.d.ts:4491](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4491)

Draw text on this image using the provided font.

##### Parameters

###### x

`number`

###### y

`number`

###### text

`string`

###### fontPath

`string`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`this`

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:4139](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4139)

Returns true if this image equals another (same dimensions and pixel data).

#### Parameters

##### other

`Image`

#### Returns

`boolean`

***

### fill()

#### Call Signature

> **fill**(`color`): `this`

Defined in: [index.d.ts:4239](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4239)

Fill this image with a color.

##### Parameters

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **fill**(`r`, `g`, `b`, `a?`): `this`

Defined in: [index.d.ts:4243](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4243)

Fill this image with a color.

##### Parameters

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

***

### filled()

#### Call Signature

> **filled**(`color`): `Image`

Defined in: [index.d.ts:4247](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4247)

Returns a copy of this image filled with a color.

##### Parameters

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **filled**(`r`, `g`, `b`, `a?`): `Image`

Defined in: [index.d.ts:4251](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4251)

Returns a copy of this image filled with a color.

##### Parameters

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

***

### findImage()

> **findImage**(`image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| `undefined`, [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Defined in: [index.d.ts:4546](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4546)

Finds the best match of an image inside this image.

Returns a `ProgressTask` that can be awaited for the result and iterated
for progress updates. Returns `undefined` if no match is found.

```ts
const match = await source.findImage(template);
if (match) {
console.log(`Found at ${match.position} with score ${match.score}`);
}
```

```ts
// Track progress while searching
const task = source.findImage(template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
}
const match = await task;
```

#### Parameters

##### image

`Image`

##### options?

[`FindImageOptions`](../interfaces/FindImageOptions.md)

#### Returns

[`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md) \| `undefined`, [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

***

### findImageAll()

> **findImageAll**(`image`, `options?`): [`ProgressTask`](../type-aliases/ProgressTask.md)\<[`Match`](../interfaces/Match.md)[], [`FindImageProgress`](../interfaces/FindImageProgress.md)\>

Defined in: [index.d.ts:4568](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4568)

Finds all occurrences of an image inside this image.

Returns a `ProgressTask` that can be awaited for an array of matches.

```ts
const matches = await source.findImageAll(template, { matchThreshold: 0.85 });
for (const match of matches) {
console.log(`Found at ${match.position}`);
}
```

```ts
// Track progress while searching
const task = source.findImageAll(template);
for await (const progress of task) {
console.log(`${progress.stage}: ${formatPercent(progress.percent)}`);
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

Defined in: [index.d.ts:4175](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4175)

Flip the image.

#### Parameters

##### flipDirection

[`FlipDirection`](../enumerations/FlipDirection.md)

#### Returns

`this`

***

### flipped()

> **flipped**(`flipDirection`): `Image`

Defined in: [index.d.ts:4179](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4179)

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

Defined in: [index.d.ts:4255](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4255)

Returns the value of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Color`](Color.md)

#### Call Signature

> **getPixel**(`x`, `y`): [`Color`](Color.md)

Defined in: [index.d.ts:4259](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4259)

Returns the value of a pixel.

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

[`Color`](Color.md)

***

### grayscale()

> **grayscale**(): `this`

Defined in: [index.d.ts:4191](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4191)

Transform this image into a grayscale.

#### Returns

`this`

***

### hueRotate()

> **hueRotate**(`value`): `this`

Defined in: [index.d.ts:4183](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4183)

Hue rotate the image.

#### Parameters

##### value

`number`

#### Returns

`this`

***

### invertColors()

> **invertColors**(): `this`

Defined in: [index.d.ts:4151](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4151)

Invert the colors of this image.

#### Returns

`this`

***

### invertedColors()

> **invertedColors**(): `Image`

Defined in: [index.d.ts:4155](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4155)

Invert the colors of this image and returns a new image.

#### Returns

`Image`

***

### resize()

> **resize**(`width`, `height`, `options?`): `this`

Defined in: [index.d.ts:4215](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4215)

Resizes this image.

#### Parameters

##### width

`number`

##### height

`number`

##### options?

[`ResizeOptions`](../interfaces/ResizeOptions.md)

#### Returns

`this`

***

### resized()

> **resized**(`width`, `height`, `options?`): `Image`

Defined in: [index.d.ts:4219](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4219)

Returns a resized version of this image.

#### Parameters

##### width

`number`

##### height

`number`

##### options?

[`ResizeOptions`](../interfaces/ResizeOptions.md)

#### Returns

`Image`

***

### rotate()

> **rotate**(`angle`, `options?`): `this`

Defined in: [index.d.ts:4167](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4167)

Rotate the image.

#### Parameters

##### angle

`number`

##### options?

[`RotationOptions`](../interfaces/RotationOptions.md)

#### Returns

`this`

***

### rotated()

> **rotated**(`angle`, `options?`): `Image`

Defined in: [index.d.ts:4171](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4171)

Rotate the image and returns a new image.

#### Parameters

##### angle

`number`

##### options?

[`RotationOptions`](../interfaces/RotationOptions.md)

#### Returns

`Image`

***

### save()

> **save**(`path`): `Promise`\<`void`\>

Defined in: [index.d.ts:4131](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4131)

Saves this image to a file. The format is inferred from the file extension.

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`void`\>

***

### setPixel()

#### Call Signature

> **setPixel**(`position`, `color`): `this`

Defined in: [index.d.ts:4263](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4263)

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

Defined in: [index.d.ts:4267](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4267)

Sets the color of a pixel.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

#### Call Signature

> **setPixel**(`x`, `y`, `color`): `this`

Defined in: [index.d.ts:4271](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4271)

Sets the color of a pixel.

##### Parameters

###### x

`number`

###### y

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`this`

#### Call Signature

> **setPixel**(`x`, `y`, `r`, `g`, `b`, `a?`): `this`

Defined in: [index.d.ts:4275](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4275)

Sets the color of a pixel.

##### Parameters

###### x

`number`

###### y

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`this`

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:4143](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4143)

Returns a string representation of this image (width, height).

#### Returns

`string`

***

### withAdjustedBrightness()

> **withAdjustedBrightness**(`value`): `Image`

Defined in: [index.d.ts:4227](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4227)

Returns a brightened or darkened version of this image.

#### Parameters

##### value

`number`

#### Returns

`Image`

***

### withAdjustedContrast()

> **withAdjustedContrast**(`value`): `Image`

Defined in: [index.d.ts:4235](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4235)

Returns a new image with an adjusted contrast.

#### Parameters

##### value

`number`

#### Returns

`Image`

***

### withCircle()

#### Call Signature

> **withCircle**(`center`, `radius`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4399](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4399)

Draw a circle on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withCircle**(`center`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4403](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4403)

Draw a circle on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### radius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withCircle**(`x`, `y`, `radius`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4407](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4407)

Draw a circle on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### radius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withCircle**(`x`, `y`, `radius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4411](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4411)

Draw a circle on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### radius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

***

### withCross()

#### Call Signature

> **withCross**(`position`, `color`): `Image`

Defined in: [index.d.ts:4303](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4303)

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

Defined in: [index.d.ts:4307](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4307)

Draw a cross on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

#### Call Signature

> **withCross**(`x`, `y`, `color`): `Image`

Defined in: [index.d.ts:4311](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4311)

Draw a cross on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withCross**(`x`, `y`, `r`, `g`, `b`, `a?`): `Image`

Defined in: [index.d.ts:4315](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4315)

Draw a cross on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

***

### withEllipse()

#### Call Signature

> **withEllipse**(`center`, `widthRadius`, `heightRadius`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4431](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4431)

Draw an ellipse on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

`number`

###### heightRadius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`center`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4435](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4435)

Draw an ellipse on a copy of this image.

##### Parameters

###### center

[`PointLike`](../type-aliases/PointLike.md)

###### widthRadius

`number`

###### heightRadius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4439](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4439)

Draw an ellipse on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### widthRadius

`number`

###### heightRadius

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withEllipse**(`x`, `y`, `widthRadius`, `heightRadius`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4443](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4443)

Draw an ellipse on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### widthRadius

`number`

###### heightRadius

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

***

### withGrayscale()

> **withGrayscale**(): `Image`

Defined in: [index.d.ts:4195](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4195)

Returns a grayscale version of this image.

#### Returns

`Image`

***

### withHueRotation()

> **withHueRotation**(`value`): `Image`

Defined in: [index.d.ts:4187](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4187)

Hue rotate the image and returns a new image.

#### Parameters

##### value

`number`

#### Returns

`Image`

***

### withImage()

#### Call Signature

> **withImage**(`position`, `image`, `options?`): `Image`

Defined in: [index.d.ts:4519](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4519)

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

Defined in: [index.d.ts:4523](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4523)

Draw another image on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

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

Defined in: [index.d.ts:4351](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4351)

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

Defined in: [index.d.ts:4355](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4355)

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

#### Call Signature

> **withLine**(`start`, `x`, `y`, `color`): `Image`

Defined in: [index.d.ts:4359](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4359)

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

`number`

###### y

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`start`, `x`, `y`, `r`, `g`, `b`, `a?`): `Image`

Defined in: [index.d.ts:4363](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4363)

Draw a line on a copy of this image.

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

`number`

###### y

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

#### Call Signature

> **withLine**(`x`, `y`, `end`, `color`): `Image`

Defined in: [index.d.ts:4367](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4367)

Draw a line on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`x`, `y`, `end`, `r`, `g`, `b`, `a?`): `Image`

Defined in: [index.d.ts:4371](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4371)

Draw a line on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

#### Call Signature

> **withLine**(`x1`, `y1`, `x2`, `y2`, `color`): `Image`

Defined in: [index.d.ts:4375](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4375)

Draw a line on a copy of this image.

##### Parameters

###### x1

`number`

###### y1

`number`

###### x2

`number`

###### y2

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

##### Returns

`Image`

#### Call Signature

> **withLine**(`x1`, `y1`, `x2`, `y2`, `r`, `g`, `b`, `a?`): `Image`

Defined in: [index.d.ts:4379](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4379)

Draw a line on a copy of this image.

##### Parameters

###### x1

`number`

###### y1

`number`

###### x2

`number`

###### y2

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

##### Returns

`Image`

***

### withRectangle()

#### Call Signature

> **withRectangle**(`rect`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4463](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4463)

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

Defined in: [index.d.ts:4467](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4467)

Draw a rectangle on a copy of this image.

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`x`, `y`, `width`, `height`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4471](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4471)

Draw a rectangle on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

#### Call Signature

> **withRectangle**(`x`, `y`, `width`, `height`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4475](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4475)

Draw a rectangle on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### width

`number`

###### height

`number`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawingOptions`](../interfaces/DrawingOptions.md)

##### Returns

`Image`

***

### withText()

#### Call Signature

> **withText**(`position`, `text`, `fontPath`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4495](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4495)

Draw text on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

`string`

###### fontPath

`string`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`Image`

#### Call Signature

> **withText**(`position`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4499](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4499)

Draw text on a copy of this image.

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

###### text

`string`

###### fontPath

`string`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`Image`

#### Call Signature

> **withText**(`x`, `y`, `text`, `fontPath`, `color`, `options?`): `Image`

Defined in: [index.d.ts:4503](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4503)

Draw text on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### text

`string`

###### fontPath

`string`

###### color

[`ColorLike`](../type-aliases/ColorLike.md)

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`Image`

#### Call Signature

> **withText**(`x`, `y`, `text`, `fontPath`, `r`, `g`, `b`, `a?`, `options?`): `Image`

Defined in: [index.d.ts:4507](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4507)

Draw text on a copy of this image.

##### Parameters

###### x

`number`

###### y

`number`

###### text

`string`

###### fontPath

`string`

###### r

`number`

###### g

`number`

###### b

`number`

###### a?

`number`

###### options?

[`DrawTextOptions`](../interfaces/DrawTextOptions.md)

##### Returns

`Image`

***

### fromBytes()

> `static` **fromBytes**(`bytes`): `Image`

Defined in: [index.d.ts:4127](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4127)

Creates a new image from raw encoded bytes (PNG, JPEG, etc.).

```ts
const bytes = await file.readAll();
const image = Image.fromBytes(bytes);
```

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Image`

***

### load()

> `static` **load**(`path`): `Promise`\<`Image`\>

Defined in: [index.d.ts:4135](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4135)

Loads an image from a file. The format is guessed from the file contents.

#### Parameters

##### path

`string`

#### Returns

`Promise`\<`Image`\>
