# Interface: Windows

Manages desktop windows: enumerate, focus, move, resize, and close windows.

```ts
// Get all windows
const allWindows = windows.all();
for (const win of allWindows) {
    println(win.title());
}
```

```ts
// Get the active window and move it
const win = windows.activeWindow();
win.setPosition(100, 100);
win.setSize(800, 600);
```

```ts
// Find and close a window by title
const matches = windows.find({ title: new Wildcard("*Notepad*") });
for (const win of matches) {
    win.close();
}
```

## Methods

### all()

> **all**(): readonly [`WindowHandle`](WindowHandle.md)[]

Returns all currently open windows.

```ts
const allWindows = windows.all();
println(`Found ${allWindows.length} windows`);
```

#### Returns

readonly [`WindowHandle`](WindowHandle.md)[]

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### active()

> **active**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Returns the currently active (focused) window, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if no window is active.

```ts
const win = windows.active();
if (win) {
  println(win.title());
}
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### foreground()

> **foreground**(): [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

Returns the currently active (focused) window, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if no window is active. Alias for `active()`.

```ts
const win = windows.foreground();
if (win) {
  println(win.title());
}
```

#### Returns

[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### find()

> **find**(`options`: [`WindowsFindOptions`](WindowsFindOptions.md)): readonly [`WindowHandle`](WindowHandle.md)[]

Finds windows matching the provided criteria.

`title` and `className` support NameLike matching (`string | Wildcard | RegExp`).

```ts
const byId = windows.find({ id: 1 });
const visibleCode = windows.find({ visible: true, title: new Wildcard("*Code*") });
const byPid = windows.find({ processId: 12345 });
const byTitle = windows.find({ title: new Wildcard("*Code*") });
const byClass = windows.find({ className: /^gnome-terminal/i });
const exact = windows.find({ title: "Calculator", className: "ApplicationFrameWindow" });
```

#### Parameters

##### options

[`WindowsFindOptions`](WindowsFindOptions.md)

<div class="options-fields">

###### id?

> `optional` **id?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by internal window ID.
When undefined, any window ID is accepted.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### processId?

> `optional` **processId?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by window process ID.
When undefined, any process ID is accepted.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### visible?

> `optional` **visible?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Match by window visibility.
When undefined, visibility is not filtered.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### title?

> `optional` **title?**: [`NameLike`](../type-aliases/NameLike.md)

Match by window title.
When undefined, title is not filtered.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### className?

> `optional` **className?**: [`NameLike`](../type-aliases/NameLike.md)

Match by window class name.
When undefined, class name is not filtered.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

readonly [`WindowHandle`](WindowHandle.md)[]

#### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### findAt()

#### Call Signature

> **findAt**(`point`: [`PointLike`](../type-aliases/PointLike.md)): readonly [`WindowHandle`](WindowHandle.md)[]

Finds windows whose rectangle contains the given screen point.

```ts
const underMouse = windows.findAt(mouse.position());
const atOrigin = windows.findAt(0, 0);
```

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

readonly [`WindowHandle`](WindowHandle.md)[]

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

#### Call Signature

> **findAt**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): readonly [`WindowHandle`](WindowHandle.md)[]

Finds windows whose rectangle contains the given screen point.

```ts
const underMouse = windows.findAt(mouse.position());
const atOrigin = windows.findAt(0, 0);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

readonly [`WindowHandle`](WindowHandle.md)[]

##### Platform

<div class="platform-badges">
<span class="platform-badge platform-badge--supported" title="Supported on Windows" aria-label="Supported on Windows"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Windows</span></span>
<span class="platform-badge platform-badge--supported" title="Supported on Linux" aria-label="Supported on Linux"><span class="platform-badge__icon" aria-hidden="true">✓</span><span class="platform-badge__label">Linux</span></span>
<span class="platform-badge platform-badge--unsupported" title="Not supported on Wayland" aria-label="Not supported on Wayland"><span class="platform-badge__icon" aria-hidden="true">✕</span><span class="platform-badge__label">Wayland</span></span>
</div>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `windows` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
