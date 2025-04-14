# Interface: RotationOptions


Options for rotating an image.

```ts
// Rotate around a custom center point
image.rotate(45, { center: new Point(10, 10) });

// You can also use a plain object for the center
image.rotate(45, { center: {x: 10, y: 10} });

// Rotate with a background color for exposed areas
image.rotate(30, { defaultColor: Color.White });
```

## Properties

### interpolation?

> `optional` **interpolation?**: [`Interpolation`](../enumerations/Interpolation.md)

Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)

#### Default Value

`Interpolation.Bilinear`

***

### center?

> `optional` **center?**: [`PointLike`](../type-aliases/PointLike.md)

Rotation center.
Defaults to the center of the image.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### defaultColor?

> `optional` **defaultColor?**: [`ColorLike`](../type-aliases/ColorLike.md)

Default color, used if the rotation triggers more pixels to be displayed

#### Default Value

`Color.Black`
