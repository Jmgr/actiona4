# Interface: Screen

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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
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
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### askScreenshot()

> <span class="async-badge">async</span> **askScreenshot**(`options?`: [`AskScreenshotOptions`](AskScreenshotOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Asks the user to interactively select a screen area and returns a
screenshot of that area, or [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if the user cancels.

On Linux the XDG Desktop Portal is used by default, which works on both
X11 and Wayland. On Windows the system Snipping Tool is used by default.
In both cases the user can cancel at any time.

```ts
const image = await screen.askScreenshot();
if (image) {
  await image.save("/tmp/selection.png");
}
```

```ts
// Force the native picker (error if unavailable)
const image = await screen.askScreenshot({ method: AskScreenshotMethod.Native });
```

```ts
// Always use the overlay selector
const image = await screen.askScreenshot({ method: AskScreenshotMethod.Overlay });
```

#### Parameters

##### options?

[`AskScreenshotOptions`](AskScreenshotOptions.md)

<div class="options-fields">

###### method?

> `optional` **method?**: [`AskScreenshotMethod`](../enumerations/AskScreenshotMethod.md)

<div class="options-fields">

###### Auto

> **Auto**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Auto`

Use the platform-default interactive screenshot picker, falling back to
the overlay selector if the native picker is unavailable.

***

###### Native

> **Native**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Native`

Use the platform native picker (XDG Desktop Portal on Linux, Snipping
Tool on Windows). Fails if the native picker is unavailable.

***

###### Overlay

> **Overlay**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`AskScreenshotMethod.Overlay`

Use the bundled overlay selector only.

###### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

</div>

Controls which capture method to use.

###### Default Value

`AskScreenshotMethod.Auto`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Image`](../classes/Image.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

***

### askRect()

> <span class="async-badge">async</span> **askRect**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Rect`](../classes/Rect.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Asks the user to interactively select a screen rectangle using the
bundled overlay selector, or returns [`null`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/null) if the user cancels.

Unlike [`screen.askScreenshot`], this returns the selected rectangle
without capturing a screenshot.

```ts
const rect = await screen.askRect();
if (rect) {
  println(`Selected: ${rect}`);
}
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Rect`](../classes/Rect.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `screen` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
