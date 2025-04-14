# Class: Font

A font loaded from a file, used to draw text on images.

```ts
const font = await Font.load("/path/to/font.ttf");
image.drawText(10, 10, "Hello", font, Color.Black);
```

## Methods

### load()

> <span class="async-badge">async</span> `static` **load**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Font`\>

Loads a font from a file.

```ts
const font = await Font.load("/path/to/font.ttf");
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Font`\>

***

### defaultFont()

> `static` **defaultFont**(): `Font`

Returns the built-in default font (DejaVu Sans).

#### Returns

`Font`

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this font.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
