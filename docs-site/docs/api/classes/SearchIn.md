# Class: SearchIn

Specifies the screen area to search within for find-image operations.

```ts
// Search the entire desktop
const match = await image.findOnScreen(SearchIn.desktop());

// Search a specific display
const display = displays.primary();
const match = await image.findOnScreen(SearchIn.display(display));

// Search a specific rectangle
const match = await image.findOnScreen(SearchIn.rect(0, 0, 1920, 1080));
```

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### desktop()

> `static` **desktop**(): `SearchIn`

Searches within the entire desktop (the bounding rectangle of all connected displays).

```ts
const match = await image.findOnScreen(SearchIn.desktop());
```

#### Returns

`SearchIn`

***

### display()

> `static` **display**(`display`: [`DisplayInfo`](../interfaces/DisplayInfo.md)): `SearchIn`

Searches within a specific display.

```ts
const display = displays.primary();
const match = await image.findOnScreen(SearchIn.display(display));
```

#### Parameters

##### display

[`DisplayInfo`](../interfaces/DisplayInfo.md)

#### Returns

`SearchIn`

***

### rect()

#### Call Signature

> `static` **rect**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): `SearchIn`

Searches within the given screen rectangle.

```ts
const match = await image.findOnScreen(SearchIn.rect(0, 0, 1920, 1080));
```

##### Parameters

###### rect

[`RectLike`](../type-aliases/RectLike.md)

##### Returns

`SearchIn`

#### Call Signature

> `static` **rect**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `width`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `height`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `SearchIn`

Searches within the given screen rectangle.

```ts
const match = await image.findOnScreen(SearchIn.rect(0, 0, 1920, 1080));
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

`SearchIn`

***

### window()

> `static` **window**(`handle`: [`WindowHandle`](../interfaces/WindowHandle.md)): `SearchIn`

Searches within the bounding rectangle of the given window.

```ts
const win = windows.activeWindow();
const match = await image.findOnScreen(SearchIn.window(win));
```

#### Parameters

##### handle

[`WindowHandle`](../interfaces/WindowHandle.md)

#### Returns

`SearchIn`
