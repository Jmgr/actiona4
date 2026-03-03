# Class: Display

A display selector resolved at capture or search time.

Use the static factory methods to create a `Display`:

```ts
// Capture a specific display
const img = await screenshot.captureDisplay(Display.primary());
const img = await screenshot.captureDisplay(Display.largest());
const img = await screenshot.captureDisplay(Display.fromId(474));
const img = await screenshot.captureDisplay(Display.fromName("HDMI-1"));
const img = await screenshot.captureDisplay(Display.fromName(new Wildcard("HDMI-*")));
const img = await screenshot.captureDisplay(Display.fromName(/HDMI-.*/));
const img = await screenshot.captureDisplay(Display.fromPoint(100, 200));
```

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### desktop()

> `static` **desktop**(): `Display`

Selects the entire desktop (the bounding rectangle of all connected displays).

```ts
const img = await screenshot.captureDisplay(Display.desktop());
```

#### Returns

`Display`

***

### fromId()

> `static` **fromId**(`id`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Display`

Selects a display by its unique numeric ID.

```ts
const img = await screenshot.captureDisplay(Display.fromId(474));
```

#### Parameters

##### id

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Display`

***

### fromName()

> `static` **fromName**(`name`: [`NameLike`](../type-aliases/NameLike.md)): `Display`

Selects a display by its friendly name.

Accepts a plain string (exact match), a `Wildcard` pattern, or a `RegExp`.
String and wildcard names are resolved at capture time (no cache required at
construction); regex names require the display cache to be available when used
with `findImage`, or will wait for it with `captureDisplay`.

```ts
const img = await screenshot.captureDisplay(Display.fromName("HDMI-1"));
const img = await screenshot.captureDisplay(Display.fromName(new Wildcard("HDMI-*")));
const img = await screenshot.captureDisplay(Display.fromName(/HDMI-.*/));
```

#### Parameters

##### name

[`NameLike`](../type-aliases/NameLike.md)

#### Returns

`Display`

***

### fromPoint()

#### Call Signature

> `static` **fromPoint**(`point`: [`PointLike`](../type-aliases/PointLike.md)): `Display`

Selects the display that contains the given point.

```ts
const img = await screenshot.captureDisplay(Display.fromPoint(100, 200));
```

##### Parameters

###### point

[`PointLike`](../type-aliases/PointLike.md)

##### Returns

`Display`

#### Call Signature

> `static` **fromPoint**(`x`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `y`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Display`

Selects the display that contains the given point.

```ts
const img = await screenshot.captureDisplay(Display.fromPoint(100, 200));
```

##### Parameters

###### x

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

###### y

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### Returns

`Display`

***

### largest()

> `static` **largest**(): `Display`

Selects the display with the largest area.

```ts
const img = await screenshot.captureDisplay(Display.largest());
```

#### Returns

`Display`

***

### primary()

> `static` **primary**(): `Display`

Selects the primary (main) display.

```ts
const img = await screenshot.captureDisplay(Display.primary());
```

#### Returns

`Display`

***

### smallest()

> `static` **smallest**(): `Display`

Selects the display with the smallest area.

```ts
const img = await screenshot.captureDisplay(Display.smallest());
```

#### Returns

`Display`
