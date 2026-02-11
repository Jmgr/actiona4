# Interface: Windows

Manages desktop windows: enumerate, focus, move, resize, and close windows.

```ts
// Get all windows
const allWindows = await windows.all();
for (const win of allWindows) {
println(await win.title());
}
```

```ts
// Get the active window and move it
const win = await windows.activeWindow();
await win.setPosition(100, 100);
await win.setSize(800, 600);
```

```ts
// Find and close a window by title
const matches = await windows.find({ title: new Wildcard("*Notepad*") });
for (const win of matches) {
await win.close();
}
```

## Methods

### activeWindow()

> **activeWindow**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md)\>\>

Returns the currently active (focused) window.

```ts
const win = await windows.activeWindow();
println(await win.title());
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md)\>\>

***

### all()

> **all**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

Returns all currently open windows.

```ts
const allWindows = await windows.all();
println(`Found ${allWindows.length} windows`);
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

***

### find()

> **find**(`options`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

Finds windows matching the provided criteria.

`title` and `className` support NameLike matching (`string | Wildcard | RegExp`).

```ts
const byId = await windows.find({ id: 1 });
const visibleCode = await windows.find({ visible: true, title: new Wildcard("*Code*") });
const byPid = await windows.find({ processId: 12345 });
const byTitle = await windows.find({ title: new Wildcard("*Code*") });
const byClass = await windows.find({ className: /^gnome-terminal/i });
const exact = await windows.find({ title: "Calculator", className: "ApplicationFrameWindow" });
```

#### Parameters

##### options

[`WindowsFindOptions`](WindowsFindOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

***

### findAt()

#### Call Signature

> **findAt**(`point`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

Finds windows whose rectangle contains the given screen point.

```ts
const underMouse = await windows.findAt(await mouse.position());
const atOrigin = await windows.findAt(0, 0);
```

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

#### Call Signature

> **findAt**(`x`, `y`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

Finds windows whose rectangle contains the given screen point.

```ts
const underMouse = await windows.findAt(await mouse.position());
const atOrigin = await windows.findAt(0, 0);
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>
