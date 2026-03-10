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

### clearEventHandles()

> **clearEventHandles**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Unregisters all event handles registered on this mouse instance.

```ts
mouse.onButton(Button.Left, () => console.println("left"));
mouse.clearEventHandles();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

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

> `optional` **duration**: [`DurationLike`](../type-aliases/DurationLike.md)

How long to hold each click, in seconds.

###### Default Value

`0`

***

###### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

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

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

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

> `optional` **delay**: [`DurationLike`](../type-aliases/DurationLike.md)

Delay between the two clicks, in seconds.

###### Default Value

`0.25`

***

###### duration?

> `optional` **duration**: [`DurationLike`](../type-aliases/DurationLike.md)

How long to hold each click, in seconds.

###### Default Value

`0`

###### Inherited from

[`ClickOptions`](ClickOptions.md).[`duration`](ClickOptions.md#duration)

***

###### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

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

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### dragAndDrop()

#### Call Signature

> <span class="async-badge">async</span> **dragAndDrop**(`start`: [`PointLike`](../type-aliases/PointLike.md), `end`: [`PointLike`](../type-aliases/PointLike.md), `options?`: [`DragOptions`](DragOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses a mouse button at `start`, moves smoothly to `end`, then releases.

```ts
// Drag an element from one position to another
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 });
```

```ts
// Drag with custom speed and right button
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 }, {
  button: Button.Right,
  speed: 500,
});
```

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### options?

[`DragOptions`](DragOptions.md)

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

Mouse button to use for dragging.

###### Default Value

`Button.Left`

***

###### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Interval in seconds

###### Default Value

`0.01`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`interval`](MoveOptions.md#interval)

***

###### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

###### Default Value

`5`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinAmplitude`](MoveOptions.md#perlinamplitude)

***

###### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

###### Default Value

`50`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinScale`](MoveOptions.md#perlinscale)

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

###### Default Value

`2000`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`speed`](MoveOptions.md#speed)

***

###### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

###### Default Value

`0`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`targetRandomness`](MoveOptions.md#targetrandomness)

***

###### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

###### Default Value

`Tween.SineOut`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`tween`](MoveOptions.md#tween)

</div>

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> <span class="async-badge">async</span> **dragAndDrop**(`start`: [`PointLike`](../type-aliases/PointLike.md), `x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DragOptions`](DragOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses a mouse button at `start`, moves smoothly to `end`, then releases.

```ts
// Drag an element from one position to another
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 });
```

```ts
// Drag with custom speed and right button
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 }, {
  button: Button.Right,
  speed: 500,
});
```

##### Parameters

###### start

[`PointLike`](../type-aliases/PointLike.md)

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DragOptions`](DragOptions.md)

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

Mouse button to use for dragging.

###### Default Value

`Button.Left`

***

###### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Interval in seconds

###### Default Value

`0.01`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`interval`](MoveOptions.md#interval)

***

###### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

###### Default Value

`5`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinAmplitude`](MoveOptions.md#perlinamplitude)

***

###### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

###### Default Value

`50`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinScale`](MoveOptions.md#perlinscale)

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

###### Default Value

`2000`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`speed`](MoveOptions.md#speed)

***

###### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

###### Default Value

`0`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`targetRandomness`](MoveOptions.md#targetrandomness)

***

###### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

###### Default Value

`Tween.SineOut`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`tween`](MoveOptions.md#tween)

</div>

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> <span class="async-badge">async</span> **dragAndDrop**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `end`: [`PointLike`](../type-aliases/PointLike.md), `options?`: [`DragOptions`](DragOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses a mouse button at `start`, moves smoothly to `end`, then releases.

```ts
// Drag an element from one position to another
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 });
```

```ts
// Drag with custom speed and right button
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 }, {
  button: Button.Right,
  speed: 500,
});
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### end

[`PointLike`](../type-aliases/PointLike.md)

###### options?

[`DragOptions`](DragOptions.md)

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

Mouse button to use for dragging.

###### Default Value

`Button.Left`

***

###### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Interval in seconds

###### Default Value

`0.01`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`interval`](MoveOptions.md#interval)

***

###### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

###### Default Value

`5`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinAmplitude`](MoveOptions.md#perlinamplitude)

***

###### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

###### Default Value

`50`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinScale`](MoveOptions.md#perlinscale)

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

###### Default Value

`2000`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`speed`](MoveOptions.md#speed)

***

###### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

###### Default Value

`0`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`targetRandomness`](MoveOptions.md#targetrandomness)

***

###### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

###### Default Value

`Tween.SineOut`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`tween`](MoveOptions.md#tween)

</div>

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> <span class="async-badge">async</span> **dragAndDrop**(`x1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y1`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `x2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y2`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`DragOptions`](DragOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Presses a mouse button at `start`, moves smoothly to `end`, then releases.

```ts
// Drag an element from one position to another
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 });
```

```ts
// Drag with custom speed and right button
await mouse.dragAndDrop({ x: 100, y: 200 }, { x: 500, y: 200 }, {
  button: Button.Right,
  speed: 500,
});
```

##### Parameters

###### x1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y1

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### x2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y2

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### options?

[`DragOptions`](DragOptions.md)

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

Mouse button to use for dragging.

###### Default Value

`Button.Left`

***

###### interval?

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

Interval in seconds

###### Default Value

`0.01`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`interval`](MoveOptions.md#interval)

***

###### perlinAmplitude?

> `optional` **perlinAmplitude**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Amplitude of the Perlin noise applied to the movement path.

###### Default Value

`5`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinAmplitude`](MoveOptions.md#perlinamplitude)

***

###### perlinScale?

> `optional` **perlinScale**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Scale of the Perlin noise applied to the movement path.

###### Default Value

`50`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`perlinScale`](MoveOptions.md#perlinscale)

***

###### speed?

> `optional` **speed**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Movement speed in pixels per second.

###### Default Value

`2000`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`speed`](MoveOptions.md#speed)

***

###### targetRandomness?

> `optional` **targetRandomness**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Random offset applied to the target position, in pixels.

###### Default Value

`0`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`targetRandomness`](MoveOptions.md#targetrandomness)

***

###### tween?

> `optional` **tween**: [`Tween`](../enumerations/Tween.md)

Easing function used for the movement.

###### Default Value

`Tween.SineOut`

###### Inherited from

[`MoveOptions`](MoveOptions.md).[`tween`](MoveOptions.md#tween)

</div>

##### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### isPressed()

> **isPressed**(`button`: [`Button`](../enumerations/Button.md)): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

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

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### measureSpeed()

> <span class="async-badge">async</span> **measureSpeed**(`options?`: [`MeasureSpeedOptions`](MeasureSpeedOptions.md)): [`Task`](../type-aliases/Task.md)\<[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)\>

Measures the mouse movement speed over a duration (in pixels per second).

#### Parameters

##### options?

[`MeasureSpeedOptions`](MeasureSpeedOptions.md)

<div class="options-fields">

###### duration?

> `optional` **duration**: [`DurationLike`](../type-aliases/DurationLike.md)

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

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

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

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

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

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

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

> `optional` **interval**: [`DurationLike`](../type-aliases/DurationLike.md)

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

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### onButton()

> **onButton**(`button`: [`Button`](../enumerations/Button.md), `callback`: [`TriggerAction`](../type-aliases/TriggerAction.md), `options?`: [`OnButtonOptions`](OnButtonOptions.md)): [`EventHandle`](EventHandle.md)

Registers a listener that fires when a mouse button is pressed.

```ts
const handle = mouse.onButton(Button.Left, () => {
  console.println("Left button pressed!");
});
// ... later:
handle.cancel();
```

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

##### callback

[`TriggerAction`](../type-aliases/TriggerAction.md)

##### options?

[`OnButtonOptions`](OnButtonOptions.md)

<div class="options-fields">

###### exclusive?

> `optional` **exclusive**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Require exactly this button and no others to be pressed simultaneously.

###### Default Value

`false`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to automatically cancel this listener when signalled.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`EventHandle`](EventHandle.md)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### onScroll()

> **onScroll**(`callback`: [`Macro`](../classes/Macro.md) \| (`length`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)) => [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void) \| [`Macro`](../classes/Macro.md) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void) \| [`Macro`](../classes/Macro.md)\>, `options?`: [`OnScrollOptions`](OnScrollOptions.md)): [`EventHandle`](EventHandle.md)

Registers a listener that fires when the mouse wheel is scrolled.

```ts
const handle = mouse.onScroll((length) => {
  console.println(`Scrolled ${length} units`);
});
// ... later:
handle.cancel();
```

```ts
// Listen for horizontal scroll only
const handle = mouse.onScroll((length) => {
  console.println(`Horizontal scroll: ${length}`);
}, { axis: Axis.Horizontal });
```

#### Parameters

##### callback

[`Macro`](../classes/Macro.md) | (`length`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)) => [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void) \| [`Macro`](../classes/Macro.md) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void) \| [`Macro`](../classes/Macro.md)\>

##### options?

[`OnScrollOptions`](OnScrollOptions.md)

<div class="options-fields">

###### axis?

> `optional` **axis**: [`Axis`](../enumerations/Axis.md)

<div class="options-fields">

###### Horizontal

> **Horizontal**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Axis.Horizontal`

***

###### Vertical

> **Vertical**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Axis.Vertical`

</div>

Axis to listen on.

###### Default Value

`Axis.Vertical`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to automatically cancel this listener when signalled.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`EventHandle`](EventHandle.md)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### position()

> **position**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>

Returns the current mouse cursor position.

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### press()

> **press**(`options?`: [`PressOptions`](PressOptions.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

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

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### release()

> **release**(`button?`: [`Button`](../enumerations/Button.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

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

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### scroll()

> **scroll**(`length`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `axis?`: [`Axis`](../enumerations/Axis.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

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

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### setPosition()

#### Call Signature

> **setPosition**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> **setPosition**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the mouse cursor position instantly (absolute coordinates).

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### setRelativePosition()

#### Call Signature

> **setRelativePosition**(`point`: [`PointLike`](../type-aliases/PointLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> **setRelativePosition**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Moves the mouse cursor by the given offset (relative coordinates).

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### waitForButton()

> <span class="async-badge">async</span> **waitForButton**(`options?`: [`WaitForButtonOptions`](WaitForButtonOptions.md)): [`Task`](../type-aliases/Task.md)\<[`Button`](../enumerations/Button.md)\>

Waits until a mouse button is pressed.

```ts
// Wait for any button press
const button = await mouse.waitForButton();
```

```ts
// Wait for left button with abort support
const controller = new AbortController();
const button = await mouse.waitForButton({
  button: Button.Left,
  signal: controller.signal
});
```

#### Parameters

##### options?

[`WaitForButtonOptions`](WaitForButtonOptions.md)

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

Mouse button to wait for. If not specified, waits for any button.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`Button`](../enumerations/Button.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### waitForScroll()

> <span class="async-badge">async</span> **waitForScroll**(`options?`: [`WaitForScrollOptions`](WaitForScrollOptions.md)): [`Task`](../type-aliases/Task.md)\<[`ScrollEvent`](ScrollEvent.md)\>

Waits until the mouse wheel is scrolled.

```ts
// Wait for any scroll event
const event = await mouse.waitForScroll();
console.println(`Scrolled ${event.length} on axis ${event.axis}`);
```

```ts
// Wait for vertical scroll with abort support
const controller = new AbortController();
const event = await mouse.waitForScroll({
  axis: Axis.Vertical,
  signal: controller.signal
});
```

#### Parameters

##### options?

[`WaitForScrollOptions`](WaitForScrollOptions.md)

<div class="options-fields">

###### axis?

> `optional` **axis**: [`Axis`](../enumerations/Axis.md)

<div class="options-fields">

###### Horizontal

> **Horizontal**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Axis.Horizontal`

***

###### Vertical

> **Vertical**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`Axis.Vertical`

</div>

Scroll axis to wait for. If not specified, waits for any axis.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`ScrollEvent`](ScrollEvent.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>
