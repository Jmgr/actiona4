# Interface: Displays

Defined in: [index.d.ts:3356](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3356)

The global displays singleton for querying connected monitors and screens.

```ts
// Get a random point across all displays
const point = await displays.randomPoint();

// Find which display contains a point
const info = await displays.fromPoint(100, 200);
if (info) console.log(info.name, info.rect);

// Find a display by friendly name
const monitor = await displays.fromName("HDMI-1");

// Get the largest or smallest display
const largest = await displays.largest();
const smallest = await displays.smallest();
```

## Methods

### fromDeviceName()

> **fromDeviceName**(`name`): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3376](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3376)

Finds a display by its device name, or `undefined` if not found.

#### Parameters

##### name

[`NameLike`](../type-aliases/NameLike.md)

#### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

***

### fromId()

> **fromId**(`id`): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3380](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3380)

Finds a display by its unique numeric ID, or `undefined` if not found.

#### Parameters

##### id

`number`

#### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

***

### fromName()

> **fromName**(`name`): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3372](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3372)

Finds a display by its friendly name (e.g. `"HDMI-1"`), or `undefined` if not found.

#### Parameters

##### name

[`NameLike`](../type-aliases/NameLike.md)

#### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

***

### fromPoint()

#### Call Signature

> **fromPoint**(`point`): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3364](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3364)

Returns the display that contains the given point, or `undefined` if none.

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

#### Call Signature

> **fromPoint**(`x`, `y`): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3368](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3368)

Returns the display that contains the given point, or `undefined` if none.

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

***

### largest()

> **largest**(): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3388](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3388)

Returns the largest display by area, or `undefined` if no displays are connected.

#### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

***

### randomPoint()

> **randomPoint**(): `Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

Defined in: [index.d.ts:3360](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3360)

Returns a random point within the bounds of all connected displays.

#### Returns

`Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

***

### smallest()

> **smallest**(): `Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>

Defined in: [index.d.ts:3384](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3384)

Returns the smallest display by area, or `undefined` if no displays are connected.

#### Returns

`Promise`\<`Readonly`\<[`DisplayInfo`](DisplayInfo.md) \| `undefined`\>\>
