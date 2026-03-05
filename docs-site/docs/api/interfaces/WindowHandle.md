# Interface: WindowHandle

A handle to a specific desktop window.

Obtained from `windows.all()` or `windows.activeWindow()`.
Provides methods to query and manipulate the window.

```ts
const win = windows.activeWindow();
println(win.title());
println(win.isVisible());
println(win.rect());
```

## Properties

### closed

> `readonly` **closed**: [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

A promise that resolves when the window is closed.

```ts
const win = windows.activeWindow();
await win.closed;
```

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

## Methods

### capture()

> <span class="async-badge">async</span> **capture**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the window's bounding rectangle.

```ts
const win = windows.activeWindow();
const image = await win.capture();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### className()

> **className**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns the window class name.

```ts
const className = win.className();
```

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### close()

> **close**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Closes this window.

```ts
win.close();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### isActive()

> **isActive**(): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns whether this window is the active (focused) window.

```ts
const active = win.isActive();
```

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### isVisible()

> **isVisible**(): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns whether this window is visible.

```ts
const visible = win.isVisible();
```

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### maximize()

> **maximize**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Maximizes this window.

```ts
win.maximize();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### minimize()

> **minimize**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Minimizes this window.

```ts
win.minimize();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### position()

> **position**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>

Returns the window position.

```ts
const pos = win.position();
println(`${pos.x}, ${pos.y}`);
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Point`](../classes/Point.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### processId()

> **processId**(): [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns the process ID of the window's owning process.

```ts
const pid = win.processId();
```

#### Returns

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### rect()

> **rect**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Rect`](../classes/Rect.md)\>

Returns the window's bounding rectangle.

```ts
const r = win.rect();
println(`${r.x}, ${r.y}, ${r.width}x${r.height}`);
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Rect`](../classes/Rect.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### setActive()

> **setActive**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Makes this window the active (focused) window.

```ts
win.setActive();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### setForeground()

> **setForeground**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Makes this window the active (focused) window. Alias for `setActive()`.

```ts
win.setForeground();
```

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### setPosition()

#### Call Signature

> **setPosition**(`position`: [`PointLike`](../type-aliases/PointLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the window position.

```ts
win.setPosition(100, 200);
win.setPosition(new Point(100, 200));
win.setPosition({x: 100, y: 200});
```

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> **setPosition**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the window position.

```ts
win.setPosition(100, 200);
win.setPosition(new Point(100, 200));
win.setPosition({x: 100, y: 200});
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### setSize()

#### Call Signature

> **setSize**(`size`: [`SizeLike`](../type-aliases/SizeLike.md)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the window size.

```ts
win.setSize(800, 600);
win.setSize(new Size(800, 600));
win.setSize({width: 800, height: 600});
```

##### Parameters

###### size

[`SizeLike`](../type-aliases/SizeLike.md)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> **setSize**(`width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the window size.

```ts
win.setSize(800, 600);
win.setSize(new Size(800, 600));
win.setSize({width: 800, height: 600});
```

##### Parameters

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### size()

> **size**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Size`](../classes/Size.md)\>

Returns the window size.

```ts
const s = win.size();
println(`${s.width}x${s.height}`);
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Size`](../classes/Size.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### title()

> **title**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns the window title.

```ts
const title = win.title();
```

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this window handle.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
