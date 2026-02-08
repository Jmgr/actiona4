# Interface: Windows

Defined in: [index.d.ts:6868](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6868)

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

> **activeWindow**(): `Promise`\<`Readonly`\<[`WindowHandle`](WindowHandle.md)\>\>

Defined in: [index.d.ts:6886](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6886)

Returns the currently active (focused) window.

```ts
const win = await windows.activeWindow();
console.log(await win.title());
```

#### Returns

`Promise`\<`Readonly`\<[`WindowHandle`](WindowHandle.md)\>\>

***

### all()

> **all**(): `Promise`\<readonly [`WindowHandle`](WindowHandle.md)[]\>

Defined in: [index.d.ts:6877](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6877)

Returns all currently open windows.

```ts
const allWindows = await windows.all();
console.log(`Found ${allWindows.length} windows`);
```

#### Returns

`Promise`\<readonly [`WindowHandle`](WindowHandle.md)[]\>
