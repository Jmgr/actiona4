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

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the display.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
