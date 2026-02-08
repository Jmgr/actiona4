# Interface: Mouse

Defined in: [index.d.ts:4743](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4743)

Controls mouse input: movement, clicking, scrolling, and position queries.

```ts
// Move and click
await mouse.move(500, 300);
await mouse.click();
```

```ts
// Right-click at a specific position
await mouse.click({ button: Button.Right, position: { x: 100, y: 200 } });
```

```ts
// Smooth movement with custom tween
await mouse.move(800, 600, {
speed: 1500,
tween: Tween.BounceOut
});
```

## Methods

### click()

> **click**(`options?`): [`Task`](../type-aliases/Task.md)\<`void`\>

Defined in: [index.d.ts:4789](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4789)

Clicks a mouse button.

#### Parameters

##### options?

[`ClickOptions`](ClickOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<`void`\>

***

### doubleClick()

> **doubleClick**(`options?`): [`Task`](../type-aliases/Task.md)\<`void`\>

Defined in: [index.d.ts:4793](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4793)

Double-clicks a mouse button.

#### Parameters

##### options?

[`DoubleClickOptions`](DoubleClickOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<`void`\>

***

### isPressed()

> **isPressed**(`button`): `Promise`\<`boolean`\>

Defined in: [index.d.ts:4748](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4748)

Returns whether a mouse button is currently pressed.

#### Parameters

##### button

[`Button`](../enumerations/Button.md)

#### Returns

`Promise`\<`boolean`\>

#### Platform

does not work on Wayland

***

### measureSpeed()

> **measureSpeed**(`options?`): [`Task`](../type-aliases/Task.md)\<`number`\>

Defined in: [index.d.ts:4761](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4761)

Measures the mouse movement speed over a duration (in pixels per second).

#### Parameters

##### options?

[`MeasureSpeedOptions`](MeasureSpeedOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<`number`\>

***

### move()

#### Call Signature

> **move**(`point`, `options?`): [`Task`](../type-aliases/Task.md)\<`void`\>

Defined in: [index.d.ts:4765](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4765)

Moves the mouse cursor smoothly to the given position.

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

###### options?

[`MoveOptions`](MoveOptions.md)

##### Returns

[`Task`](../type-aliases/Task.md)\<`void`\>

#### Call Signature

> **move**(`x`, `y`, `options?`): [`Task`](../type-aliases/Task.md)\<`void`\>

Defined in: [index.d.ts:4769](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4769)

Moves the mouse cursor smoothly to the given position.

##### Parameters

###### x

`number`

###### y

`number`

###### options?

[`MoveOptions`](MoveOptions.md)

##### Returns

[`Task`](../type-aliases/Task.md)\<`void`\>

***

### position()

> **position**(): `Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

Defined in: [index.d.ts:4757](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4757)

Returns the current mouse cursor position.

#### Returns

`Promise`\<`Readonly`\<[`Point`](../classes/Point.md)\>\>

#### Platform

does not work on Wayland

***

### press()

> **press**(`options?`): `Promise`\<`void`\>

Defined in: [index.d.ts:4797](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4797)

Presses and holds a mouse button.

#### Parameters

##### options?

[`PressOptions`](PressOptions.md)

#### Returns

`Promise`\<`void`\>

***

### release()

> **release**(`button?`): `Promise`\<`void`\>

Defined in: [index.d.ts:4801](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4801)

Releases a mouse button.

#### Parameters

##### button?

[`Button`](../enumerations/Button.md)

#### Returns

`Promise`\<`void`\>

***

### scroll()

> **scroll**(`length`, `axis?`): `Promise`\<`void`\>

Defined in: [index.d.ts:4752](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4752)

Scrolls the mouse wheel by the given amount.

#### Parameters

##### length

`number`

##### axis?

[`Axis`](../enumerations/Axis.md)

#### Returns

`Promise`\<`void`\>

***

### setPosition()

#### Call Signature

> **setPosition**(`point`): `Promise`\<`void`\>

Defined in: [index.d.ts:4773](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4773)

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

`Promise`\<`void`\>

#### Call Signature

> **setPosition**(`x`, `y`): `Promise`\<`void`\>

Defined in: [index.d.ts:4777](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4777)

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`Promise`\<`void`\>

***

### setRelativePosition()

#### Call Signature

> **setRelativePosition**(`point`): `Promise`\<`void`\>

Defined in: [index.d.ts:4781](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4781)

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

`Promise`\<`void`\>

#### Call Signature

> **setRelativePosition**(`x`, `y`): `Promise`\<`void`\>

Defined in: [index.d.ts:4785](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L4785)

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### x

`number`

###### y

`number`

##### Returns

`Promise`\<`void`\>
