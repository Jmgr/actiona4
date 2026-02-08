# Interface: Windows

Manages desktop windows: enumerate, focus, move, resize, and close windows.

```ts
// Get all windows
const allWindows = await windows.all();
for (const win of allWindows) {
console.log(await win.title());
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
const allWindows = await windows.all();
for (const win of allWindows) {
if ((await win.title()).includes("Notepad")) {
await win.close();
}
}
```

## Methods

### activeWindow()

> **activeWindow**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md)\>\>

Returns the currently active (focused) window.

```ts
const win = await windows.activeWindow();
console.log(await win.title());
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`WindowHandle`](WindowHandle.md)\>\>

***

### all()

> **all**(): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>

Returns all currently open windows.

```ts
const allWindows = await windows.all();
console.log(`Found ${allWindows.length} windows`);
```

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`WindowHandle`](WindowHandle.md)[]\>
