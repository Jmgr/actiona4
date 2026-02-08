# Interface: DisplayInfo

Defined in: [index.d.ts:3407](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3407)

Information about a connected display, including its name, geometry,
rotation, scale factor, and refresh rate.

```ts
const info = await displays.fromName("HDMI-1");
if (info) {
console.log(info.friendlyName, info.rect, formatFrequency(info.frequency));
console.log("Primary:", info.isPrimary);
}
```

## Properties

### frequency

> `readonly` **frequency**: `number`

Defined in: [index.d.ts:3443](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3443)

The display refresh rate in Hz.

***

### friendlyName

> `readonly` **friendlyName**: `string`

Defined in: [index.d.ts:3419](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3419)

The display friendly name (e.g. `"HDMI-1"`).

***

### heightMm

> `readonly` **heightMm**: `number`

Defined in: [index.d.ts:3431](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3431)

The physical height of the display in millimeters.

***

### id

> `readonly` **id**: `number`

Defined in: [index.d.ts:3411](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3411)

Unique numeric identifier for this display.

***

### isPrimary

> `readonly` **isPrimary**: `boolean`

Defined in: [index.d.ts:3447](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3447)

Whether this is the primary (main) display.

***

### name

> `readonly` **name**: `string`

Defined in: [index.d.ts:3415](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3415)

The display device name (e.g. `"DP-1"`).

***

### rect

> `readonly` **rect**: `Readonly`\<[`Rect`](../classes/Rect.md)\>

Defined in: [index.d.ts:3423](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3423)

The display rectangle (position and size in pixels).

***

### rotation

> `readonly` **rotation**: `number`

Defined in: [index.d.ts:3435](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3435)

The display rotation in clock-wise degrees (0, 90, 180, or 270).

***

### scaleFactor

> `readonly` **scaleFactor**: `number`

Defined in: [index.d.ts:3439](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3439)

The display's pixel scale factor (e.g. `2.0` for HiDPI/Retina).

***

### widthMm

> `readonly` **widthMm**: `number`

Defined in: [index.d.ts:3427](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3427)

The physical width of the display in millimeters.
