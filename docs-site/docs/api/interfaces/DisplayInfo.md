# Interface: DisplayInfo

Information about a connected display, including its name, geometry,
rotation, scale factor, and refresh rate.

```ts
const info = await displays.fromName("HDMI-1");
if (info) {
    println(info.friendlyName, info.rect, formatFrequency(info.frequency));
    println("Primary:", info.isPrimary);
}
```

## Properties

### frequency

> `readonly` **frequency**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The display refresh rate in Hz.

***

### friendlyName

> `readonly` **friendlyName**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The display friendly name (e.g. `"HDMI-1"`).

***

### heightMm

> `readonly` **heightMm**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The physical height of the display in millimeters.

***

### id

> `readonly` **id**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Unique numeric identifier for this display.

***

### isPrimary

> `readonly` **isPrimary**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether this is the primary (main) display.

***

### name

> `readonly` **name**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The display device name (e.g. `"DP-1"`).

***

### rect

> `readonly` **rect**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Rect`](../classes/Rect.md)\>

The display rectangle (position and size in pixels).

***

### rotation

> `readonly` **rotation**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The display rotation in clock-wise degrees (0, 90, 180, or 270).

***

### scaleFactor

> `readonly` **scaleFactor**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The display's pixel scale factor (e.g. `2.0` for HiDPI/Retina).

***

### widthMm

> `readonly` **widthMm**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The physical width of the display in millimeters.

## Methods

### toGlobal()

#### Call Signature

> **toGlobal**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`Point`](../classes/Point.md)

Converts a display-local point to global desktop coordinates.

The inverse of `toLocal`: adds this display's top-left offset so the
point can be used with mouse, keyboard, or capture APIs that expect
global coordinates.

```ts
const display = displays.primary();
// A point at (100, 50) within the display image:
const global = display.toGlobal(100, 50);
```

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Point`](../classes/Point.md)

#### Call Signature

> **toGlobal**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Point`](../classes/Point.md)

Converts a display-local point to global desktop coordinates.

The inverse of `toLocal`: adds this display's top-left offset so the
point can be used with mouse, keyboard, or capture APIs that expect
global coordinates.

```ts
const display = displays.primary();
// A point at (100, 50) within the display image:
const global = display.toGlobal(100, 50);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Point`](../classes/Point.md)

***

### toLocal()

#### Call Signature

> **toLocal**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`Point`](../classes/Point.md)

Converts a global desktop point to display-local coordinates.

The result is the position relative to this display's top-left corner,
in the same logical-pixel unit used for mouse coordinates and `rect`.
DPI scaling and rotation are already normalised into the desktop
coordinate system by the OS, so no additional transform is needed.

```ts
const display = displays.primary();
// After finding something at global coordinate (1980, 50):
const local = display.toLocal(1980, 50);
println(local.x, local.y); // position within the display
```

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Point`](../classes/Point.md)

#### Call Signature

> **toLocal**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Point`](../classes/Point.md)

Converts a global desktop point to display-local coordinates.

The result is the position relative to this display's top-left corner,
in the same logical-pixel unit used for mouse coordinates and `rect`.
DPI scaling and rotation are already normalised into the desktop
coordinate system by the OS, so no additional transform is needed.

```ts
const display = displays.primary();
// After finding something at global coordinate (1980, 50):
const local = display.toLocal(1980, 50);
println(local.x, local.y); // position within the display
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Point`](../classes/Point.md)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the display.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
