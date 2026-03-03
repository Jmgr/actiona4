# Class: SearchIn

Specifies the screen area to search within for find-image operations.

```ts
// Search the entire desktop
const match = await screenshot.findImage(SearchIn.desktop(), template);

// Search a specific display
const match = await screenshot.findImage(SearchIn.display(Display.primary()), template);

// Search a specific rectangle
const match = await screenshot.findImage(SearchIn.rect(0, 0, 1920, 1080), template);
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
const match = await screenshot.findImage(SearchIn.desktop(), template);
```

#### Returns

`SearchIn`

***

### display()

> `static` **display**(`display`: [`Display`](Display.md)): `SearchIn`

Searches within a specific display identified by a `Display` selector.

```ts
const match = await screenshot.findImage(SearchIn.display(Display.primary()), template);
```

#### Parameters

##### display

[`Display`](Display.md)

#### Returns

`SearchIn`

***

### rect()

#### Call Signature

> `static` **rect**(`rect`: [`RectLike`](../type-aliases/RectLike.md)): `SearchIn`

Searches within the given screen rectangle.

```ts
const match = await screenshot.findImage(SearchIn.rect(0, 0, 1920, 1080), template);
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
const match = await screenshot.findImage(SearchIn.rect(0, 0, 1920, 1080), template);
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
const match = await screenshot.findImage(SearchIn.window(win), template);
```

#### Parameters

##### handle

[`WindowHandle`](../interfaces/WindowHandle.md)

#### Returns

`SearchIn`
