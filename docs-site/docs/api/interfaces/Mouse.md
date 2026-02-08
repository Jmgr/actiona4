# Interface: Mouse

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

> **click**(`options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Clicks a mouse button.

#### Parameters

##### options?

[`ClickOptions`](ClickOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### doubleClick()

> **doubleClick**(`options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Double-clicks a mouse button.

#### Parameters

##### options?

[`DoubleClickOptions`](DoubleClickOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### isPressed()

> **isPressed**(`button`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns whether a mouse button is currently pressed.

#### Parameters

##### button

[`Button`](../enumerations/Button.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

#### Platform

does not work on Wayland

***

### measureSpeed()

> **measureSpeed**(`options?`): [`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Measures the mouse movement speed over a duration (in pixels per second).

#### Parameters

##### options?

[`MeasureSpeedOptions`](MeasureSpeedOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

***

### move()

#### Call Signature

> **move**(`point`, `options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor smoothly to the given position.

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

###### options?

[`MoveOptions`](MoveOptions.md)

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Call Signature

> **move**(`x`, `y`, `options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor smoothly to the given position.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`MoveOptions`](MoveOptions.md)

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### position()

> **position**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

Returns the current mouse cursor position.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

#### Platform

does not work on Wayland

***

### press()

> **press**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses and holds a mouse button.

#### Parameters

##### options?

[`PressOptions`](PressOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### release()

> **release**(`button?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Releases a mouse button.

#### Parameters

##### button?

[`Button`](../enumerations/Button.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### scroll()

> **scroll**(`length`, `axis?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Scrolls the mouse wheel by the given amount.

#### Parameters

##### length

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### axis?

[`Axis`](../enumerations/Axis.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setPosition()

#### Call Signature

> **setPosition**(`point`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Call Signature

> **setPosition**(`x`, `y`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setRelativePosition()

#### Call Signature

> **setRelativePosition**(`point`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Call Signature

> **setRelativePosition**(`x`, `y`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
