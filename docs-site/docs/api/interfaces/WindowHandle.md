# Interface: WindowHandle

Defined in: [index.d.ts:6906](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6906)

A handle to a specific desktop window.

Obtained from `windows.all()` or `windows.activeWindow()`.
Provides methods to query and manipulate the window.

```ts
const win = await windows.activeWindow();
console.log(await win.title());
console.log(await win.isVisible());
console.log(await win.rect());
```

## Methods

### className()

> **className**(): `Promise`\<`string`\>

Defined in: [index.d.ts:6930](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6930)

Returns the window class name.

```ts
const className = await win.className();
```

#### Returns

`Promise`\<`string`\>

***

### close()

> **close**(): `Promise`\<`void`\>

Defined in: [index.d.ts:6938](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6938)

Closes this window.

```ts
await win.close();
```

#### Returns

`Promise`\<`void`\>

***

### isActive()

> **isActive**(): `Promise`\<`boolean`\>

Defined in: [index.d.ts:7045](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7045)

Returns whether this window is the active (focused) window.

```ts
const active = await win.isActive();
```

#### Returns

`Promise`\<`boolean`\>

***

### isVisible()

> **isVisible**(): `Promise`\<`boolean`\>

Defined in: [index.d.ts:6914](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6914)

Returns whether this window is visible.

```ts
const visible = await win.isVisible();
```

#### Returns

`Promise`\<`boolean`\>

***

### maximize()

> **maximize**(): `Promise`\<`void`\>

Defined in: [index.d.ts:6979](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6979)

Maximizes this window.

```ts
await win.maximize();
```

#### Returns

`Promise`\<`void`\>

***

### minimize()

> **minimize**(): `Promise`\<`void`\>

Defined in: [index.d.ts:6971](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6971)

Minimizes this window.

```ts
await win.minimize();
```

#### Returns

`Promise`\<`void`\>

***

### position()

> **position**(): `Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

Defined in: [index.d.ts:7008](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7008)

Returns the window position.

```ts
const pos = await win.position();
console.log(`${pos.x}, ${pos.y}`);
```

#### Returns

`Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

***

### processId()

> **processId**(): `Promise`\<`number`\>

Defined in: [index.d.ts:6946](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6946)

Returns the process ID of the window's owning process.

```ts
const pid = await win.processId();
```

#### Returns

`Promise`\<`number`\>

***

### rect()

> **rect**(): `Promise`\<`Readonly`\<[`Rect`](../classes/Rect.md)\>\>

Defined in: [index.d.ts:6955](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6955)

Returns the window's bounding rectangle.

```ts
const r = await win.rect();
console.log(`${r.x}, ${r.y}, ${r.width}x${r.height}`);
```

#### Returns

`Promise`\<`Readonly`\<[`Rect`](../classes/Rect.md)\>\>

***

### setActive()

> **setActive**(): `Promise`\<`void`\>

Defined in: [index.d.ts:6963](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6963)

Makes this window the active (focused) window.

```ts
await win.setActive();
```

#### Returns

`Promise`\<`void`\>

***

### setPosition()

#### Call Signature

> **setPosition**(`position`): `Promise`\<`void`\>

Defined in: [index.d.ts:6989](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6989)

Sets the window position.

```ts
await win.setPosition(100, 200);
await win.setPosition(new Point(100, 200));
await win.setPosition({x: 100, y: 200});
```

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

`Promise`\<`void`\>

#### Call Signature

> **setPosition**(`x`, `y`): `Promise`\<`void`\>

Defined in: [index.d.ts:6999](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6999)

Sets the window position.

```ts
await win.setPosition(100, 200);
await win.setPosition(new Point(100, 200));
await win.setPosition({x: 100, y: 200});
```

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`Promise`\<`void`\>

***

### setSize()

#### Call Signature

> **setSize**(`size`): `Promise`\<`void`\>

Defined in: [index.d.ts:7018](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7018)

Sets the window size.

```ts
await win.setSize(800, 600);
await win.setSize(new Size(800, 600));
await win.setSize({width: 800, height: 600});
```

##### Parameters

###### size

[`SizeLike`](../type-aliases/SizeLike.md)

##### Returns

`Promise`\<`void`\>

#### Call Signature

> **setSize**(`width`, `height`): `Promise`\<`void`\>

Defined in: [index.d.ts:7028](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7028)

Sets the window size.

```ts
await win.setSize(800, 600);
await win.setSize(new Size(800, 600));
await win.setSize({width: 800, height: 600});
```

##### Parameters

###### width

`number`

###### height

`number`

##### Returns

`Promise`\<`void`\>

***

### size()

> **size**(): `Promise`\<`Readonly`\<[`Size`](../classes/Size.md)\>\>

Defined in: [index.d.ts:7037](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7037)

Returns the window size.

```ts
const s = await win.size();
console.log(`${s.width}x${s.height}`);
```

#### Returns

`Promise`\<`Readonly`\<[`Size`](../classes/Size.md)\>\>

***

### title()

> **title**(): `Promise`\<`string`\>

Defined in: [index.d.ts:6922](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6922)

Returns the window title.

```ts
const title = await win.title();
```

#### Returns

`Promise`\<`string`\>
