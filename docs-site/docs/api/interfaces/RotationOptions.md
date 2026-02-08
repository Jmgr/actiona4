# Interface: RotationOptions

Defined in: [index.d.ts:3882](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3882)

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

Defined in: [index.d.ts:3892](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3892)

Rotation center

#### Default Value

```ts
image center
```

***

### defaultColor?

> `optional` **defaultColor**: [`Color`](../classes/Color.md)

Defined in: [index.d.ts:3897](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3897)

Default color, used if the rotation triggers more pixels to be displayed

#### Default Value

`Color.Black`

***

### interpolation?

> `optional` **interpolation**: [`Interpolation`](../enumerations/Interpolation.md)

Defined in: [index.d.ts:3887](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3887)

Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)

#### Default Value

`Interpolation.Bilinear`
