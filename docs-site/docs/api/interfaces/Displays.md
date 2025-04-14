# Interface: Displays

The global displays singleton for querying connected monitors and screens.

```ts
// Get the primary display and convert a global coordinate to display-local
const display = displays.primary();
const local = display.toLocal(globalX, globalY);

// Find which display contains a point
const info = displays.fromPoint(100, 200);
if (info) println(info.name, info.rect);

// Find a display by friendly name
const monitor = displays.fromName("HDMI-1");

// Get the largest or smallest display
const largest = displays.largest();
const smallest = displays.smallest();

// Get a random point across all displays
const point = await displays.randomPoint();
```

## Methods

### randomPoint()

> <span class="async-badge">async</span> **randomPoint**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

Returns a random point within the bounds of all connected displays.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

***

### primary()

> **primary**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the primary display, or throws if no primary display is found.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### fromPoint()

#### Call Signature

> **fromPoint**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Returns the display that contains the given point, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if none.

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

#### Call Signature

> **fromPoint**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Returns the display that contains the given point, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if none.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### fromName()

> **fromName**(`name`: [`NameLike`](../type-aliases/NameLike.md)): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Finds a display by its friendly name (e.g. `"HDMI-1"`), or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if not found.

#### Parameters

##### name

[`NameLike`](../type-aliases/NameLike.md)

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### fromDeviceName()

> **fromDeviceName**(`name`: [`NameLike`](../type-aliases/NameLike.md)): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Finds a display by its device name, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if not found.

#### Parameters

##### name

[`NameLike`](../type-aliases/NameLike.md)

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### fromId()

> **fromId**(`id`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Finds a display by its unique numeric ID, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if not found.

#### Parameters

##### id

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### smallest()

> **smallest**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the smallest display by area, or throws if no displays are connected.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### largest()

> **largest**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the largest display by area, or throws if no displays are connected.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### leftmost()

> **leftmost**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the display furthest to the left (minimum left edge), or throws if none.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### rightmost()

> **rightmost**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the display furthest to the right (maximum right edge), or throws if none.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### topmost()

> **topmost**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the display furthest to the top (minimum top edge), or throws if none.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### bottommost()

> **bottommost**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the display furthest to the bottom (maximum bottom edge), or throws if none.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### center()

> **center**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

Returns the display whose center is closest to the center of the desktop, or throws if none.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`DisplayInfo`](DisplayInfo.md)\>

***

### all()

> **all**(): readonly [`DisplayInfo`](DisplayInfo.md)[]

Returns all displays.

#### Returns

readonly [`DisplayInfo`](DisplayInfo.md)[]

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `displays` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
