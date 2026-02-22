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

> <span class="async-badge">async</span> **click**(`options?`: [`ClickOptions`](ClickOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Clicks a mouse button.

#### Parameters

##### options?

[`ClickOptions`](ClickOptions.md)

<div class="options-fields">

###### amount?

> `optional` **amount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Number of times to click.

###### Default Value

`1`

***

###### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

<div class="options-fields">

###### Back

> **Back**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Back button
`Button.Back`

***

###### Forward

> **Forward**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Forward button
`Button.Forward`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Left button
`Button.Left`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Middle button
`Button.Middle`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Right button
`Button.Right`

</div>

Mouse button to press.

###### Default Value

`Button.Left`

###### Inherited from

[`PressOptions`](PressOptions.md).[`button`](PressOptions.md#button)

***

###### duration?

> `optional` **duration**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How long to hold each click, in seconds.

###### Default Value

`0`

***

###### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delay between consecutive clicks, in seconds.

###### Default Value

`0`

***

###### position?

> `optional` **position**: [`Point`](../classes/Point.md)

Position to move the cursor to before pressing.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

###### Inherited from

[`PressOptions`](PressOptions.md).[`position`](PressOptions.md#position)

***

###### relativePosition?

> `optional` **relativePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the position is relative to the current cursor position.

###### Default Value

`false`

###### Inherited from

[`PressOptions`](PressOptions.md).[`relativePosition`](PressOptions.md#relativeposition)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the click.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### doubleClick()

> <span class="async-badge">async</span> **doubleClick**(`options?`: [`DoubleClickOptions`](DoubleClickOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Double-clicks a mouse button.

#### Parameters

##### options?

[`DoubleClickOptions`](DoubleClickOptions.md)

<div class="options-fields">

###### amount?

> `optional` **amount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Number of times to click.

###### Default Value

`1`

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`amount`](ClickOptions.md#amount)

***

###### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

<div class="options-fields">

###### Back

> **Back**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Back button
`Button.Back`

***

###### Forward

> **Forward**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Forward button
`Button.Forward`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Left button
`Button.Left`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Middle button
`Button.Middle`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Right button
`Button.Right`

</div>

Mouse button to press.

###### Default Value

`Button.Left`

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`button`](ClickOptions.md#button)

***

###### delay?

> `optional` **delay**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delay between the two clicks, in seconds.

###### Default Value

`0.25`

***

###### duration?

> `optional` **duration**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

How long to hold each click, in seconds.

###### Default Value

`0`

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`duration`](ClickOptions.md#duration)

***

###### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Delay between consecutive clicks, in seconds.

###### Default Value

`0`

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`interval`](ClickOptions.md#interval)

***

###### position?

> `optional` **position**: [`Point`](../classes/Point.md)

Position to move the cursor to before pressing.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`position`](ClickOptions.md#position)

***

###### relativePosition?

> `optional` **relativePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the position is relative to the current cursor position.

###### Default Value

`false`

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`relativePosition`](ClickOptions.md#relativeposition)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the click.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`signal`](ClickOptions.md#signal)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### isPressed()

> <span class="async-badge">async</span> **isPressed**(`button`: [`Button`](../enumerations/Button.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

Returns whether a mouse button is currently pressed.

#### Parameters

##### button

[`Button`](../enumerations/Button.md)

<div class="options-fields">

###### Back

> **Back**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Back button
`Button.Back`

***

###### Forward

> **Forward**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Forward button
`Button.Forward`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Left button
`Button.Left`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Middle button
`Button.Middle`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Right button
`Button.Right`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)\>

#### Platform

does not work on Wayland

***

### measureSpeed()

> <span class="async-badge">async</span> **measureSpeed**(`options?`: [`MeasureSpeedOptions`](MeasureSpeedOptions.md)): [`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Measures the mouse movement speed over a duration (in pixels per second).

#### Parameters

##### options?

[`MeasureSpeedOptions`](MeasureSpeedOptions.md)

<div class="options-fields">

###### duration?

> `optional` **duration**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Measurement duration.

###### Default Value

`2s`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the measurement.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

***

### move()

#### Call Signature

> <span class="async-badge">async</span> **move**(`point`: [`PointLike`](../type-aliases/PointLike.md), `options?`: [`MoveOptions`](MoveOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor smoothly to the given position.

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

###### options?

[`MoveOptions`](MoveOptions.md)

<div class="options-fields">

###### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Interval in seconds

###### Default Value

`0.01`

***

###### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

###### Default Value

`5`

***

###### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

###### Default Value

`50`

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

###### Default Value

`2000`

***

###### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

###### Default Value

`0`

***

###### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

###### Default Value

`Tween.SineOut`

</div>

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Call Signature

> <span class="async-badge">async</span> **move**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`MoveOptions`](MoveOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor smoothly to the given position.

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`MoveOptions`](MoveOptions.md)

<div class="options-fields">

###### interval?

> `optional` **interval**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Interval in seconds

###### Default Value

`0.01`

***

###### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

###### Default Value

`5`

***

###### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

###### Default Value

`50`

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

###### Default Value

`2000`

***

###### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

###### Default Value

`0`

***

###### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

###### Default Value

`Tween.SineOut`

</div>

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### position()

> <span class="async-badge">async</span> **position**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

Returns the current mouse cursor position.

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>\>

#### Platform

does not work on Wayland

***

### press()

> <span class="async-badge">async</span> **press**(`options?`: [`PressOptions`](PressOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses and holds a mouse button.

#### Parameters

##### options?

[`PressOptions`](PressOptions.md)

<div class="options-fields">

###### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

<div class="options-fields">

###### Back

> **Back**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Back button
`Button.Back`

***

###### Forward

> **Forward**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Forward button
`Button.Forward`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Left button
`Button.Left`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Middle button
`Button.Middle`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Right button
`Button.Right`

</div>

Mouse button to press.

###### Default Value

`Button.Left`

***

###### position?

> `optional` **position**: [`Point`](../classes/Point.md)

Position to move the cursor to before pressing.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### relativePosition?

> `optional` **relativePosition**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the position is relative to the current cursor position.

###### Default Value

`false`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### release()

> <span class="async-badge">async</span> **release**(`button?`: [`Button`](../enumerations/Button.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Releases a mouse button.

#### Parameters

##### button?

[`Button`](../enumerations/Button.md)

<div class="options-fields">

###### Back

> **Back**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Back button
`Button.Back`

***

###### Forward

> **Forward**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Forward button
`Button.Forward`

***

###### Left

> **Left**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Left button
`Button.Left`

***

###### Middle

> **Middle**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Middle button
`Button.Middle`

***

###### Right

> **Right**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Right button
`Button.Right`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### scroll()

> <span class="async-badge">async</span> **scroll**(`length`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `axis?`: [`Axis`](../enumerations/Axis.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Scrolls the mouse wheel by the given amount.

#### Parameters

##### length

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### axis?

[`Axis`](../enumerations/Axis.md)

<div class="options-fields">

###### Horizontal

> **Horizontal**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Axis.Horizontal`

***

###### Vertical

> **Vertical**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Axis.Vertical`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### setPosition()

#### Call Signature

> <span class="async-badge">async</span> **setPosition**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Call Signature

> <span class="async-badge">async</span> **setPosition**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

> <span class="async-badge">async</span> **setRelativePosition**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

#### Call Signature

> <span class="async-badge">async</span> **setRelativePosition**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
