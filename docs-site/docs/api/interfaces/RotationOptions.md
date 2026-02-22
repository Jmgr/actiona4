# Interface: RotationOptions

**`Expand`**

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

### center?

> `optional` **center**: [`Point`](../classes/Point.md)

Rotation center

#### Default Value

```ts
image center
```

***

### defaultColor?

> `optional` **defaultColor**: [`Color`](../classes/Color.md)

Default color, used if the rotation triggers more pixels to be displayed

#### Default Value

`Color.Black`

***

### interpolation?

> `optional` **interpolation**: [`Interpolation`](../enumerations/Interpolation.md)

Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)

#### Default Value

`Interpolation.Bilinear`
