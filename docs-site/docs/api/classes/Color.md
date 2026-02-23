# Class: Color

An RGBA color with 8-bit channels.

Can be constructed from individual r/g/b/a values, or by using one of the
many named color constants (CSS colors).

```ts
// Create from RGB (alpha defaults to 255)
const red = new Color(255, 0, 0);

// Create from RGBA
const semiTransparent = new Color(255, 0, 0, 128);

// Use a named constant
const blue = Color.Blue;

// Read and modify channels
const c = new Color(10, 20, 30);
c.r = 100;
println(c); // "Color(100, 20, 30, 255)"

// Compare colors
Color.Red.equals(new Color(255, 0, 0)); // true

// Clone a color
const copy = Color.Red.clone();
```

```js
let c = new Color(128, 255, 255, 255);
```

## Constructors

### Constructor

> **new Color**(`r`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `g`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `b`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `a?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): `Color`

Constructor with three color channels and an alpha channel.

#### Parameters

##### r

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### g

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### b

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

##### a?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

#### Returns

`Color`

### Constructor

> **new Color**(`c`: [`ColorLike`](../type-aliases/ColorLike.md)): `Color`

Constructor with anything Color-like.

#### Parameters

##### c

[`ColorLike`](../type-aliases/ColorLike.md)

#### Returns

`Color`

## Properties

### a

> **a**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Alpha (should be between 0-255)

***

### b

> **b**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Blue (should be between 0-255)

***

### g

> **g**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Green (should be between 0-255)

***

### r

> **r**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Red (should be between 0-255)

***

### AliceBlue

> `readonly` `static` **AliceBlue**: `Color`

#F0F8FFFF

***

### AntiqueWhite

> `readonly` `static` **AntiqueWhite**: `Color`

#FAEBD7FF

***

### Aqua

> `readonly` `static` **Aqua**: `Color`

#00FFFFFF

***

### Aquamarine

> `readonly` `static` **Aquamarine**: `Color`

#7FFFD4FF

***

### Azure

> `readonly` `static` **Azure**: `Color`

#F0FFFFFF

***

### Beige

> `readonly` `static` **Beige**: `Color`

#F5F5DCFF

***

### Bisque

> `readonly` `static` **Bisque**: `Color`

#FFE4C4FF

***

### Black

> `readonly` `static` **Black**: `Color`

#000000FF

***

### BlanchedAlmond

> `readonly` `static` **BlanchedAlmond**: `Color`

#FFEBCDFF

***

### Blue

> `readonly` `static` **Blue**: `Color`

#0000FFFF

***

### BlueViolet

> `readonly` `static` **BlueViolet**: `Color`

#8A2BE2FF

***

### Brown

> `readonly` `static` **Brown**: `Color`

#A52A2AFF

***

### BurlyWood

> `readonly` `static` **BurlyWood**: `Color`

#DEB887FF

***

### CadetBlue

> `readonly` `static` **CadetBlue**: `Color`

#5F9EA0FF

***

### Chartreuse

> `readonly` `static` **Chartreuse**: `Color`

#7FFF00FF

***

### Chocolate

> `readonly` `static` **Chocolate**: `Color`

#D2691EFF

***

### Coral

> `readonly` `static` **Coral**: `Color`

#FF7F50FF

***

### CornflowerBlue

> `readonly` `static` **CornflowerBlue**: `Color`

#6495EDFF

***

### Cornsilk

> `readonly` `static` **Cornsilk**: `Color`

#FFF8DCFF

***

### Crimson

> `readonly` `static` **Crimson**: `Color`

#DC143CFF

***

### Cyan

> `readonly` `static` **Cyan**: `Color`

#00FFFFFF

***

### DarkBlue

> `readonly` `static` **DarkBlue**: `Color`

#00008BFF

***

### DarkCyan

> `readonly` `static` **DarkCyan**: `Color`

#008B8BFF

***

### DarkGoldenRod

> `readonly` `static` **DarkGoldenRod**: `Color`

#B8860BFF

***

### DarkGray

> `readonly` `static` **DarkGray**: `Color`

#A9A9A9FF

***

### DarkGreen

> `readonly` `static` **DarkGreen**: `Color`

#006400FF

***

### DarkKhaki

> `readonly` `static` **DarkKhaki**: `Color`

#BDB76BFF

***

### DarkMagenta

> `readonly` `static` **DarkMagenta**: `Color`

#8B008BFF

***

### DarkOliveGreen

> `readonly` `static` **DarkOliveGreen**: `Color`

#556B2FFF

***

### DarkOrange

> `readonly` `static` **DarkOrange**: `Color`

#FF8C00FF

***

### DarkOrchid

> `readonly` `static` **DarkOrchid**: `Color`

#9932CCFF

***

### DarkRed

> `readonly` `static` **DarkRed**: `Color`

#8B0000FF

***

### DarkSalmon

> `readonly` `static` **DarkSalmon**: `Color`

#E9967AFF

***

### DarkSeaGreen

> `readonly` `static` **DarkSeaGreen**: `Color`

#8FBC8FFF

***

### DarkSlateBlue

> `readonly` `static` **DarkSlateBlue**: `Color`

#483D8BFF

***

### DarkSlateGray

> `readonly` `static` **DarkSlateGray**: `Color`

#2F4F4FFF

***

### DarkTurquoise

> `readonly` `static` **DarkTurquoise**: `Color`

#00CED1FF

***

### DarkViolet

> `readonly` `static` **DarkViolet**: `Color`

#9400D3FF

***

### DeepPink

> `readonly` `static` **DeepPink**: `Color`

#FF1493FF

***

### DeepSkyBlue

> `readonly` `static` **DeepSkyBlue**: `Color`

#00BFFFFF

***

### DimGray

> `readonly` `static` **DimGray**: `Color`

#696969FF

***

### DodgerBlue

> `readonly` `static` **DodgerBlue**: `Color`

#1E90FFFF

***

### Firebrick

> `readonly` `static` **Firebrick**: `Color`

#B22222FF

***

### FloralWhite

> `readonly` `static` **FloralWhite**: `Color`

#FFFAF0FF

***

### ForestGreen

> `readonly` `static` **ForestGreen**: `Color`

#228B22FF

***

### Fuchsia

> `readonly` `static` **Fuchsia**: `Color`

#FF00FFFF

***

### Gainsboro

> `readonly` `static` **Gainsboro**: `Color`

#DCDCDCFF

***

### GhostWhite

> `readonly` `static` **GhostWhite**: `Color`

#F8F8FFFF

***

### Gold

> `readonly` `static` **Gold**: `Color`

#FFD700FF

***

### GoldenRod

> `readonly` `static` **GoldenRod**: `Color`

#DAA520FF

***

### Gray

> `readonly` `static` **Gray**: `Color`

#808080FF

***

### Green

> `readonly` `static` **Green**: `Color`

#008000FF

***

### GreenYellow

> `readonly` `static` **GreenYellow**: `Color`

#ADFF2FFF

***

### HoneyDew

> `readonly` `static` **HoneyDew**: `Color`

#F0FFF0FF

***

### HotPink

> `readonly` `static` **HotPink**: `Color`

#FF69B4FF

***

### IndianRed

> `readonly` `static` **IndianRed**: `Color`

#CD5C5CFF

***

### Indigo

> `readonly` `static` **Indigo**: `Color`

#4B0082FF

***

### Ivory

> `readonly` `static` **Ivory**: `Color`

#FFFFF0FF

***

### Khaki

> `readonly` `static` **Khaki**: `Color`

#F0E68CFF

***

### Lavender

> `readonly` `static` **Lavender**: `Color`

#E6E6FAFF

***

### LavenderBlush

> `readonly` `static` **LavenderBlush**: `Color`

#FFF0F5FF

***

### LawnGreen

> `readonly` `static` **LawnGreen**: `Color`

#7CFC00FF

***

### LemonChiffon

> `readonly` `static` **LemonChiffon**: `Color`

#FFFACDFF

***

### LightBlue

> `readonly` `static` **LightBlue**: `Color`

#ADD8E6FF

***

### LightCoral

> `readonly` `static` **LightCoral**: `Color`

#F08080FF

***

### LightCyan

> `readonly` `static` **LightCyan**: `Color`

#E0FFFFFF

***

### LightGoldenRodYellow

> `readonly` `static` **LightGoldenRodYellow**: `Color`

#FAFAD2FF

***

### LightGray

> `readonly` `static` **LightGray**: `Color`

#D3D3D3FF

***

### LightGreen

> `readonly` `static` **LightGreen**: `Color`

#90EE90FF

***

### LightPink

> `readonly` `static` **LightPink**: `Color`

#FFB6C1FF

***

### LightSalmon

> `readonly` `static` **LightSalmon**: `Color`

#FFA07AFF

***

### LightSeaGreen

> `readonly` `static` **LightSeaGreen**: `Color`

#20B2AAFF

***

### LightSkyBlue

> `readonly` `static` **LightSkyBlue**: `Color`

#87CEFAFF

***

### LightSlateGray

> `readonly` `static` **LightSlateGray**: `Color`

#778899FF

***

### LightSteelBlue

> `readonly` `static` **LightSteelBlue**: `Color`

#B0C4DEFF

***

### LightYellow

> `readonly` `static` **LightYellow**: `Color`

#FFFFE0FF

***

### Lime

> `readonly` `static` **Lime**: `Color`

#00FF00FF

***

### LimeGreen

> `readonly` `static` **LimeGreen**: `Color`

#32CD32FF

***

### Linen

> `readonly` `static` **Linen**: `Color`

#FAF0E6FF

***

### Magenta

> `readonly` `static` **Magenta**: `Color`

#FF00FFFF

***

### Maroon

> `readonly` `static` **Maroon**: `Color`

#800000FF

***

### MediumAquaMarine

> `readonly` `static` **MediumAquaMarine**: `Color`

#66CDAAFF

***

### MediumBlue

> `readonly` `static` **MediumBlue**: `Color`

#0000CDFF

***

### MediumOrchid

> `readonly` `static` **MediumOrchid**: `Color`

#BA55D3FF

***

### MediumPurple

> `readonly` `static` **MediumPurple**: `Color`

#9370DBFF

***

### MediumSeaGreen

> `readonly` `static` **MediumSeaGreen**: `Color`

#3CB371FF

***

### MediumSlateBlue

> `readonly` `static` **MediumSlateBlue**: `Color`

#7B68EEFF

***

### MediumSpringGreen

> `readonly` `static` **MediumSpringGreen**: `Color`

#00FA9AFF

***

### MediumTurquoise

> `readonly` `static` **MediumTurquoise**: `Color`

#48D1CCFF

***

### MediumVioletRed

> `readonly` `static` **MediumVioletRed**: `Color`

#C71585FF

***

### MidnightBlue

> `readonly` `static` **MidnightBlue**: `Color`

#191970FF

***

### MintCream

> `readonly` `static` **MintCream**: `Color`

#F5FFFAFF

***

### MistyRose

> `readonly` `static` **MistyRose**: `Color`

#FFE4E1FF

***

### Moccasin

> `readonly` `static` **Moccasin**: `Color`

#FFE4B5FF

***

### NavajoWhite

> `readonly` `static` **NavajoWhite**: `Color`

#FFDEADFF

***

### Navy

> `readonly` `static` **Navy**: `Color`

#000080FF

***

### OldLace

> `readonly` `static` **OldLace**: `Color`

#FDF5E6FF

***

### Olive

> `readonly` `static` **Olive**: `Color`

#808000FF

***

### OliveDrab

> `readonly` `static` **OliveDrab**: `Color`

#6B8E23FF

***

### Orange

> `readonly` `static` **Orange**: `Color`

#FFA500FF

***

### OrangeRed

> `readonly` `static` **OrangeRed**: `Color`

#FF4500FF

***

### Orchid

> `readonly` `static` **Orchid**: `Color`

#DA70D6FF

***

### PaleGoldenRod

> `readonly` `static` **PaleGoldenRod**: `Color`

#EEE8AAFF

***

### PaleGreen

> `readonly` `static` **PaleGreen**: `Color`

#98FB98FF

***

### PaleTurquoise

> `readonly` `static` **PaleTurquoise**: `Color`

#AFEEEEFF

***

### PaleVioletRed

> `readonly` `static` **PaleVioletRed**: `Color`

#DB7093FF

***

### PapayaWhip

> `readonly` `static` **PapayaWhip**: `Color`

#FFEFD5FF

***

### PeachPuff

> `readonly` `static` **PeachPuff**: `Color`

#FFDAB9FF

***

### Peru

> `readonly` `static` **Peru**: `Color`

#CD853FFF

***

### Pink

> `readonly` `static` **Pink**: `Color`

#FFC0CBFF

***

### Plum

> `readonly` `static` **Plum**: `Color`

#DDA0DDFF

***

### PowderBlue

> `readonly` `static` **PowderBlue**: `Color`

#B0E0E6FF

***

### Purple

> `readonly` `static` **Purple**: `Color`

#800080FF

***

### RebeccaPurple

> `readonly` `static` **RebeccaPurple**: `Color`

#663399FF

***

### Red

> `readonly` `static` **Red**: `Color`

#FF0000FF

***

### RosyBrown

> `readonly` `static` **RosyBrown**: `Color`

#BC8F8FFF

***

### RoyalBlue

> `readonly` `static` **RoyalBlue**: `Color`

#4169E1FF

***

### SaddleBrown

> `readonly` `static` **SaddleBrown**: `Color`

#8B4513FF

***

### Salmon

> `readonly` `static` **Salmon**: `Color`

#FA8072FF

***

### SandyBrown

> `readonly` `static` **SandyBrown**: `Color`

#F4A460FF

***

### SeaGreen

> `readonly` `static` **SeaGreen**: `Color`

#2E8B57FF

***

### SeaShell

> `readonly` `static` **SeaShell**: `Color`

#FFF5EEFF

***

### Sienna

> `readonly` `static` **Sienna**: `Color`

#A0522DFF

***

### Silver

> `readonly` `static` **Silver**: `Color`

#C0C0C0FF

***

### SkyBlue

> `readonly` `static` **SkyBlue**: `Color`

#87CEEBFF

***

### SlateBlue

> `readonly` `static` **SlateBlue**: `Color`

#6A5ACDFF

***

### SlateGray

> `readonly` `static` **SlateGray**: `Color`

#708090FF

***

### Snow

> `readonly` `static` **Snow**: `Color`

#FFFAFAFF

***

### SpringGreen

> `readonly` `static` **SpringGreen**: `Color`

#00FF7FFF

***

### SteelBlue

> `readonly` `static` **SteelBlue**: `Color`

#4682B4FF

***

### Tan

> `readonly` `static` **Tan**: `Color`

#D2B48CFF

***

### Teal

> `readonly` `static` **Teal**: `Color`

#008080FF

***

### Thistle

> `readonly` `static` **Thistle**: `Color`

#D8BFD8FF

***

### Tomato

> `readonly` `static` **Tomato**: `Color`

#FF6347FF

***

### Transparent

> `readonly` `static` **Transparent**: `Color`

#00000000

***

### Turquoise

> `readonly` `static` **Turquoise**: `Color`

#40E0D0FF

***

### Violet

> `readonly` `static` **Violet**: `Color`

#EE82EEFF

***

### Wheat

> `readonly` `static` **Wheat**: `Color`

#F5DEB3FF

***

### White

> `readonly` `static` **White**: `Color`

#FFFFFFFF

***

### WhiteSmoke

> `readonly` `static` **WhiteSmoke**: `Color`

#F5F5F5FF

***

### Yellow

> `readonly` `static` **Yellow**: `Color`

#FFFF00FF

***

### YellowGreen

> `readonly` `static` **YellowGreen**: `Color`

#9ACD32FF

## Methods

### clone()

> **clone**(): `Color`

Returns a copy of this color.

#### Returns

`Color`

***

### equals()

> **equals**(`other`: `Color`): [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Returns `true` if both colors have the same r, g, b, and a values.

```ts
Color.Red.equals(new Color(255, 0, 0)); // true
```

#### Parameters

##### other

`Color`

#### Returns

[`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the color: `"Color(r: R, g: G, b: B, a: A)"`.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
