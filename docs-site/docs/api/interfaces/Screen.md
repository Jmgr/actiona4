# Interface: Screen

Screen capture and image search.

Provides methods to capture the entire desktop, a specific display, a screen
region, or a single pixel.

```ts
const image = await screen.captureDesktop();
println(image.size());
```

```ts
const display = displays.primary();
const image = await screen.captureDisplay(display);
println(image.size());
```

```ts
const pixel = await screen.capturePixel(100, 100);
println(pixel);
```

## Methods

### captureDesktop()

> <span class="async-badge">async</span> **captureDesktop**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the entire desktop.

```ts
const image = await screen.captureDesktop();
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### captureDisplay()

> <span class="async-badge">async</span> **captureDisplay**(`display`: [`DisplayInfo`](DisplayInfo.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the given display.

```ts
const image = await screen.captureDisplay(displays.primary());
const image = await screen.captureDisplay(displays.fromId(474));
const image = await screen.captureDisplay(displays.largest());
```

#### Parameters

##### display

[`DisplayInfo`](DisplayInfo.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### capturePixel()

#### Call Signature

> <span class="async-badge">async</span> **capturePixel**(`position`: [`PointLike`](../type-aliases/PointLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

Captures the color of a single pixel on screen.

```ts
const color = await screen.capturePixel(100, 200);
println(color);
```

##### Parameters

###### position

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> <span class="async-badge">async</span> **capturePixel**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

Captures the color of a single pixel on screen.

```ts
const color = await screen.capturePixel(100, 200);
println(color);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Color`](../classes/Color.md)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### captureRect()

#### Call Signature

> <span class="async-badge">async</span> **captureRect**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of a screen rectangle.

```ts
const image = await screen.captureRect(0, 0, 1920, 1080);
```

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> <span class="async-badge">async</span> **captureRect**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of a screen rectangle.

```ts
const image = await screen.captureRect(0, 0, 1920, 1080);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### width

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### height

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### captureWindow()

> <span class="async-badge">async</span> **captureWindow**(`handle`: [`WindowHandle`](WindowHandle.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

Captures a screenshot of the bounding rectangle of the given window.

```ts
const win = windows.activeWindow();
const image = await screen.captureWindow(win);
```

#### Parameters

##### handle

[`WindowHandle`](WindowHandle.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--unsupported" title="Does not work on Wayland"><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
