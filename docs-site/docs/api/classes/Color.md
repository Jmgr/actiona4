# Class: Color

Defined in: [index.d.ts:2502](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2502)

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
console.log(c.toString()); // "(100, 20, 30, 255)"

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

> **new Color**(`r`, `g`, `b`, `a?`): `Color`

Defined in: [index.d.ts:3090](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3090)

Constructor with three color channels and an alpha channel.

#### Parameters

##### r

`number`

##### g

`number`

##### b

`number`

##### a?

`number`

#### Returns

`Color`

### Constructor

> **new Color**(`c`): `Color`

Defined in: [index.d.ts:3094](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3094)

Constructor with anything Color-like.

#### Parameters

##### c

[`ColorLike`](../type-aliases/ColorLike.md)

#### Returns

`Color`

## Properties

### a

> **a**: `number`

Defined in: [index.d.ts:3086](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3086)

Alpha (should be between 0-255)

***

### b

> **b**: `number`

Defined in: [index.d.ts:3082](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3082)

Blue (should be between 0-255)

***

### g

> **g**: `number`

Defined in: [index.d.ts:3078](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3078)

Green (should be between 0-255)

***

### r

> **r**: `number`

Defined in: [index.d.ts:3074](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3074)

Red (should be between 0-255)

***

### AliceBlue

> `readonly` `static` **AliceBlue**: `Color`

Defined in: [index.d.ts:2530](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2530)

#F0F8FFFF

***

### AntiqueWhite

> `readonly` `static` **AntiqueWhite**: `Color`

Defined in: [index.d.ts:2534](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2534)

#FAEBD7FF

***

### Aqua

> `readonly` `static` **Aqua**: `Color`

Defined in: [index.d.ts:2538](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2538)

#00FFFFFF

***

### Aquamarine

> `readonly` `static` **Aquamarine**: `Color`

Defined in: [index.d.ts:2542](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2542)

#7FFFD4FF

***

### Azure

> `readonly` `static` **Azure**: `Color`

Defined in: [index.d.ts:2546](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2546)

#F0FFFFFF

***

### Beige

> `readonly` `static` **Beige**: `Color`

Defined in: [index.d.ts:2550](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2550)

#F5F5DCFF

***

### Bisque

> `readonly` `static` **Bisque**: `Color`

Defined in: [index.d.ts:2554](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2554)

#FFE4C4FF

***

### Black

> `readonly` `static` **Black**: `Color`

Defined in: [index.d.ts:2522](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2522)

#000000FF

***

### BlanchedAlmond

> `readonly` `static` **BlanchedAlmond**: `Color`

Defined in: [index.d.ts:2558](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2558)

#FFEBCDFF

***

### Blue

> `readonly` `static` **Blue**: `Color`

Defined in: [index.d.ts:2514](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2514)

#0000FFFF

***

### BlueViolet

> `readonly` `static` **BlueViolet**: `Color`

Defined in: [index.d.ts:2562](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2562)

#8A2BE2FF

***

### Brown

> `readonly` `static` **Brown**: `Color`

Defined in: [index.d.ts:2566](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2566)

#A52A2AFF

***

### BurlyWood

> `readonly` `static` **BurlyWood**: `Color`

Defined in: [index.d.ts:2570](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2570)

#DEB887FF

***

### CadetBlue

> `readonly` `static` **CadetBlue**: `Color`

Defined in: [index.d.ts:2574](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2574)

#5F9EA0FF

***

### Chartreuse

> `readonly` `static` **Chartreuse**: `Color`

Defined in: [index.d.ts:2578](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2578)

#7FFF00FF

***

### Chocolate

> `readonly` `static` **Chocolate**: `Color`

Defined in: [index.d.ts:2582](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2582)

#D2691EFF

***

### Coral

> `readonly` `static` **Coral**: `Color`

Defined in: [index.d.ts:2586](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2586)

#FF7F50FF

***

### CornflowerBlue

> `readonly` `static` **CornflowerBlue**: `Color`

Defined in: [index.d.ts:2590](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2590)

#6495EDFF

***

### Cornsilk

> `readonly` `static` **Cornsilk**: `Color`

Defined in: [index.d.ts:2594](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2594)

#FFF8DCFF

***

### Crimson

> `readonly` `static` **Crimson**: `Color`

Defined in: [index.d.ts:2598](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2598)

#DC143CFF

***

### Cyan

> `readonly` `static` **Cyan**: `Color`

Defined in: [index.d.ts:2602](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2602)

#00FFFFFF

***

### DarkBlue

> `readonly` `static` **DarkBlue**: `Color`

Defined in: [index.d.ts:2606](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2606)

#00008BFF

***

### DarkCyan

> `readonly` `static` **DarkCyan**: `Color`

Defined in: [index.d.ts:2610](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2610)

#008B8BFF

***

### DarkGoldenRod

> `readonly` `static` **DarkGoldenRod**: `Color`

Defined in: [index.d.ts:2614](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2614)

#B8860BFF

***

### DarkGray

> `readonly` `static` **DarkGray**: `Color`

Defined in: [index.d.ts:2618](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2618)

#A9A9A9FF

***

### DarkGreen

> `readonly` `static` **DarkGreen**: `Color`

Defined in: [index.d.ts:2622](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2622)

#006400FF

***

### DarkKhaki

> `readonly` `static` **DarkKhaki**: `Color`

Defined in: [index.d.ts:2626](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2626)

#BDB76BFF

***

### DarkMagenta

> `readonly` `static` **DarkMagenta**: `Color`

Defined in: [index.d.ts:2630](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2630)

#8B008BFF

***

### DarkOliveGreen

> `readonly` `static` **DarkOliveGreen**: `Color`

Defined in: [index.d.ts:2634](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2634)

#556B2FFF

***

### DarkOrange

> `readonly` `static` **DarkOrange**: `Color`

Defined in: [index.d.ts:2638](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2638)

#FF8C00FF

***

### DarkOrchid

> `readonly` `static` **DarkOrchid**: `Color`

Defined in: [index.d.ts:2642](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2642)

#9932CCFF

***

### DarkRed

> `readonly` `static` **DarkRed**: `Color`

Defined in: [index.d.ts:2646](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2646)

#8B0000FF

***

### DarkSalmon

> `readonly` `static` **DarkSalmon**: `Color`

Defined in: [index.d.ts:2650](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2650)

#E9967AFF

***

### DarkSeaGreen

> `readonly` `static` **DarkSeaGreen**: `Color`

Defined in: [index.d.ts:2654](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2654)

#8FBC8FFF

***

### DarkSlateBlue

> `readonly` `static` **DarkSlateBlue**: `Color`

Defined in: [index.d.ts:2658](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2658)

#483D8BFF

***

### DarkSlateGray

> `readonly` `static` **DarkSlateGray**: `Color`

Defined in: [index.d.ts:2662](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2662)

#2F4F4FFF

***

### DarkTurquoise

> `readonly` `static` **DarkTurquoise**: `Color`

Defined in: [index.d.ts:2666](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2666)

#00CED1FF

***

### DarkViolet

> `readonly` `static` **DarkViolet**: `Color`

Defined in: [index.d.ts:2670](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2670)

#9400D3FF

***

### DeepPink

> `readonly` `static` **DeepPink**: `Color`

Defined in: [index.d.ts:2674](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2674)

#FF1493FF

***

### DeepSkyBlue

> `readonly` `static` **DeepSkyBlue**: `Color`

Defined in: [index.d.ts:2678](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2678)

#00BFFFFF

***

### DimGray

> `readonly` `static` **DimGray**: `Color`

Defined in: [index.d.ts:2682](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2682)

#696969FF

***

### DodgerBlue

> `readonly` `static` **DodgerBlue**: `Color`

Defined in: [index.d.ts:2686](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2686)

#1E90FFFF

***

### Firebrick

> `readonly` `static` **Firebrick**: `Color`

Defined in: [index.d.ts:2690](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2690)

#B22222FF

***

### FloralWhite

> `readonly` `static` **FloralWhite**: `Color`

Defined in: [index.d.ts:2694](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2694)

#FFFAF0FF

***

### ForestGreen

> `readonly` `static` **ForestGreen**: `Color`

Defined in: [index.d.ts:2698](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2698)

#228B22FF

***

### Fuchsia

> `readonly` `static` **Fuchsia**: `Color`

Defined in: [index.d.ts:2702](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2702)

#FF00FFFF

***

### Gainsboro

> `readonly` `static` **Gainsboro**: `Color`

Defined in: [index.d.ts:2706](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2706)

#DCDCDCFF

***

### GhostWhite

> `readonly` `static` **GhostWhite**: `Color`

Defined in: [index.d.ts:2710](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2710)

#F8F8FFFF

***

### Gold

> `readonly` `static` **Gold**: `Color`

Defined in: [index.d.ts:2714](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2714)

#FFD700FF

***

### GoldenRod

> `readonly` `static` **GoldenRod**: `Color`

Defined in: [index.d.ts:2718](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2718)

#DAA520FF

***

### Gray

> `readonly` `static` **Gray**: `Color`

Defined in: [index.d.ts:2722](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2722)

#808080FF

***

### Green

> `readonly` `static` **Green**: `Color`

Defined in: [index.d.ts:2510](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2510)

#008000FF

***

### GreenYellow

> `readonly` `static` **GreenYellow**: `Color`

Defined in: [index.d.ts:2726](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2726)

#ADFF2FFF

***

### HoneyDew

> `readonly` `static` **HoneyDew**: `Color`

Defined in: [index.d.ts:2730](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2730)

#F0FFF0FF

***

### HotPink

> `readonly` `static` **HotPink**: `Color`

Defined in: [index.d.ts:2734](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2734)

#FF69B4FF

***

### IndianRed

> `readonly` `static` **IndianRed**: `Color`

Defined in: [index.d.ts:2738](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2738)

#CD5C5CFF

***

### Indigo

> `readonly` `static` **Indigo**: `Color`

Defined in: [index.d.ts:2742](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2742)

#4B0082FF

***

### Ivory

> `readonly` `static` **Ivory**: `Color`

Defined in: [index.d.ts:2746](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2746)

#FFFFF0FF

***

### Khaki

> `readonly` `static` **Khaki**: `Color`

Defined in: [index.d.ts:2750](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2750)

#F0E68CFF

***

### Lavender

> `readonly` `static` **Lavender**: `Color`

Defined in: [index.d.ts:2754](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2754)

#E6E6FAFF

***

### LavenderBlush

> `readonly` `static` **LavenderBlush**: `Color`

Defined in: [index.d.ts:2758](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2758)

#FFF0F5FF

***

### LawnGreen

> `readonly` `static` **LawnGreen**: `Color`

Defined in: [index.d.ts:2762](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2762)

#7CFC00FF

***

### LemonChiffon

> `readonly` `static` **LemonChiffon**: `Color`

Defined in: [index.d.ts:2766](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2766)

#FFFACDFF

***

### LightBlue

> `readonly` `static` **LightBlue**: `Color`

Defined in: [index.d.ts:2770](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2770)

#ADD8E6FF

***

### LightCoral

> `readonly` `static` **LightCoral**: `Color`

Defined in: [index.d.ts:2774](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2774)

#F08080FF

***

### LightCyan

> `readonly` `static` **LightCyan**: `Color`

Defined in: [index.d.ts:2778](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2778)

#E0FFFFFF

***

### LightGoldenRodYellow

> `readonly` `static` **LightGoldenRodYellow**: `Color`

Defined in: [index.d.ts:2782](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2782)

#FAFAD2FF

***

### LightGray

> `readonly` `static` **LightGray**: `Color`

Defined in: [index.d.ts:2786](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2786)

#D3D3D3FF

***

### LightGreen

> `readonly` `static` **LightGreen**: `Color`

Defined in: [index.d.ts:2790](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2790)

#90EE90FF

***

### LightPink

> `readonly` `static` **LightPink**: `Color`

Defined in: [index.d.ts:2794](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2794)

#FFB6C1FF

***

### LightSalmon

> `readonly` `static` **LightSalmon**: `Color`

Defined in: [index.d.ts:2798](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2798)

#FFA07AFF

***

### LightSeaGreen

> `readonly` `static` **LightSeaGreen**: `Color`

Defined in: [index.d.ts:2802](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2802)

#20B2AAFF

***

### LightSkyBlue

> `readonly` `static` **LightSkyBlue**: `Color`

Defined in: [index.d.ts:2806](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2806)

#87CEFAFF

***

### LightSlateGray

> `readonly` `static` **LightSlateGray**: `Color`

Defined in: [index.d.ts:2810](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2810)

#778899FF

***

### LightSteelBlue

> `readonly` `static` **LightSteelBlue**: `Color`

Defined in: [index.d.ts:2814](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2814)

#B0C4DEFF

***

### LightYellow

> `readonly` `static` **LightYellow**: `Color`

Defined in: [index.d.ts:2818](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2818)

#FFFFE0FF

***

### Lime

> `readonly` `static` **Lime**: `Color`

Defined in: [index.d.ts:2822](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2822)

#00FF00FF

***

### LimeGreen

> `readonly` `static` **LimeGreen**: `Color`

Defined in: [index.d.ts:2826](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2826)

#32CD32FF

***

### Linen

> `readonly` `static` **Linen**: `Color`

Defined in: [index.d.ts:2830](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2830)

#FAF0E6FF

***

### Magenta

> `readonly` `static` **Magenta**: `Color`

Defined in: [index.d.ts:2834](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2834)

#FF00FFFF

***

### Maroon

> `readonly` `static` **Maroon**: `Color`

Defined in: [index.d.ts:2838](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2838)

#800000FF

***

### MediumAquaMarine

> `readonly` `static` **MediumAquaMarine**: `Color`

Defined in: [index.d.ts:2842](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2842)

#66CDAAFF

***

### MediumBlue

> `readonly` `static` **MediumBlue**: `Color`

Defined in: [index.d.ts:2846](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2846)

#0000CDFF

***

### MediumOrchid

> `readonly` `static` **MediumOrchid**: `Color`

Defined in: [index.d.ts:2850](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2850)

#BA55D3FF

***

### MediumPurple

> `readonly` `static` **MediumPurple**: `Color`

Defined in: [index.d.ts:2854](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2854)

#9370DBFF

***

### MediumSeaGreen

> `readonly` `static` **MediumSeaGreen**: `Color`

Defined in: [index.d.ts:2858](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2858)

#3CB371FF

***

### MediumSlateBlue

> `readonly` `static` **MediumSlateBlue**: `Color`

Defined in: [index.d.ts:2862](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2862)

#7B68EEFF

***

### MediumSpringGreen

> `readonly` `static` **MediumSpringGreen**: `Color`

Defined in: [index.d.ts:2866](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2866)

#00FA9AFF

***

### MediumTurquoise

> `readonly` `static` **MediumTurquoise**: `Color`

Defined in: [index.d.ts:2870](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2870)

#48D1CCFF

***

### MediumVioletRed

> `readonly` `static` **MediumVioletRed**: `Color`

Defined in: [index.d.ts:2874](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2874)

#C71585FF

***

### MidnightBlue

> `readonly` `static` **MidnightBlue**: `Color`

Defined in: [index.d.ts:2878](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2878)

#191970FF

***

### MintCream

> `readonly` `static` **MintCream**: `Color`

Defined in: [index.d.ts:2882](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2882)

#F5FFFAFF

***

### MistyRose

> `readonly` `static` **MistyRose**: `Color`

Defined in: [index.d.ts:2886](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2886)

#FFE4E1FF

***

### Moccasin

> `readonly` `static` **Moccasin**: `Color`

Defined in: [index.d.ts:2890](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2890)

#FFE4B5FF

***

### NavajoWhite

> `readonly` `static` **NavajoWhite**: `Color`

Defined in: [index.d.ts:2894](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2894)

#FFDEADFF

***

### Navy

> `readonly` `static` **Navy**: `Color`

Defined in: [index.d.ts:2898](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2898)

#000080FF

***

### OldLace

> `readonly` `static` **OldLace**: `Color`

Defined in: [index.d.ts:2902](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2902)

#FDF5E6FF

***

### Olive

> `readonly` `static` **Olive**: `Color`

Defined in: [index.d.ts:2906](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2906)

#808000FF

***

### OliveDrab

> `readonly` `static` **OliveDrab**: `Color`

Defined in: [index.d.ts:2910](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2910)

#6B8E23FF

***

### Orange

> `readonly` `static` **Orange**: `Color`

Defined in: [index.d.ts:2914](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2914)

#FFA500FF

***

### OrangeRed

> `readonly` `static` **OrangeRed**: `Color`

Defined in: [index.d.ts:2918](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2918)

#FF4500FF

***

### Orchid

> `readonly` `static` **Orchid**: `Color`

Defined in: [index.d.ts:2922](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2922)

#DA70D6FF

***

### PaleGoldenRod

> `readonly` `static` **PaleGoldenRod**: `Color`

Defined in: [index.d.ts:2926](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2926)

#EEE8AAFF

***

### PaleGreen

> `readonly` `static` **PaleGreen**: `Color`

Defined in: [index.d.ts:2930](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2930)

#98FB98FF

***

### PaleTurquoise

> `readonly` `static` **PaleTurquoise**: `Color`

Defined in: [index.d.ts:2934](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2934)

#AFEEEEFF

***

### PaleVioletRed

> `readonly` `static` **PaleVioletRed**: `Color`

Defined in: [index.d.ts:2938](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2938)

#DB7093FF

***

### PapayaWhip

> `readonly` `static` **PapayaWhip**: `Color`

Defined in: [index.d.ts:2942](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2942)

#FFEFD5FF

***

### PeachPuff

> `readonly` `static` **PeachPuff**: `Color`

Defined in: [index.d.ts:2946](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2946)

#FFDAB9FF

***

### Peru

> `readonly` `static` **Peru**: `Color`

Defined in: [index.d.ts:2950](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2950)

#CD853FFF

***

### Pink

> `readonly` `static` **Pink**: `Color`

Defined in: [index.d.ts:2954](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2954)

#FFC0CBFF

***

### Plum

> `readonly` `static` **Plum**: `Color`

Defined in: [index.d.ts:2958](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2958)

#DDA0DDFF

***

### PowderBlue

> `readonly` `static` **PowderBlue**: `Color`

Defined in: [index.d.ts:2962](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2962)

#B0E0E6FF

***

### Purple

> `readonly` `static` **Purple**: `Color`

Defined in: [index.d.ts:2966](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2966)

#800080FF

***

### RebeccaPurple

> `readonly` `static` **RebeccaPurple**: `Color`

Defined in: [index.d.ts:2970](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2970)

#663399FF

***

### Red

> `readonly` `static` **Red**: `Color`

Defined in: [index.d.ts:2506](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2506)

#FF0000FF

***

### RosyBrown

> `readonly` `static` **RosyBrown**: `Color`

Defined in: [index.d.ts:2974](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2974)

#BC8F8FFF

***

### RoyalBlue

> `readonly` `static` **RoyalBlue**: `Color`

Defined in: [index.d.ts:2978](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2978)

#4169E1FF

***

### SaddleBrown

> `readonly` `static` **SaddleBrown**: `Color`

Defined in: [index.d.ts:2982](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2982)

#8B4513FF

***

### Salmon

> `readonly` `static` **Salmon**: `Color`

Defined in: [index.d.ts:2986](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2986)

#FA8072FF

***

### SandyBrown

> `readonly` `static` **SandyBrown**: `Color`

Defined in: [index.d.ts:2990](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2990)

#F4A460FF

***

### SeaGreen

> `readonly` `static` **SeaGreen**: `Color`

Defined in: [index.d.ts:2994](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2994)

#2E8B57FF

***

### SeaShell

> `readonly` `static` **SeaShell**: `Color`

Defined in: [index.d.ts:2998](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2998)

#FFF5EEFF

***

### Sienna

> `readonly` `static` **Sienna**: `Color`

Defined in: [index.d.ts:3002](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3002)

#A0522DFF

***

### Silver

> `readonly` `static` **Silver**: `Color`

Defined in: [index.d.ts:3006](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3006)

#C0C0C0FF

***

### SkyBlue

> `readonly` `static` **SkyBlue**: `Color`

Defined in: [index.d.ts:3010](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3010)

#87CEEBFF

***

### SlateBlue

> `readonly` `static` **SlateBlue**: `Color`

Defined in: [index.d.ts:3014](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3014)

#6A5ACDFF

***

### SlateGray

> `readonly` `static` **SlateGray**: `Color`

Defined in: [index.d.ts:3018](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3018)

#708090FF

***

### Snow

> `readonly` `static` **Snow**: `Color`

Defined in: [index.d.ts:3022](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3022)

#FFFAFAFF

***

### SpringGreen

> `readonly` `static` **SpringGreen**: `Color`

Defined in: [index.d.ts:3026](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3026)

#00FF7FFF

***

### SteelBlue

> `readonly` `static` **SteelBlue**: `Color`

Defined in: [index.d.ts:3030](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3030)

#4682B4FF

***

### Tan

> `readonly` `static` **Tan**: `Color`

Defined in: [index.d.ts:3034](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3034)

#D2B48CFF

***

### Teal

> `readonly` `static` **Teal**: `Color`

Defined in: [index.d.ts:3038](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3038)

#008080FF

***

### Thistle

> `readonly` `static` **Thistle**: `Color`

Defined in: [index.d.ts:3042](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3042)

#D8BFD8FF

***

### Tomato

> `readonly` `static` **Tomato**: `Color`

Defined in: [index.d.ts:3046](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3046)

#FF6347FF

***

### Transparent

> `readonly` `static` **Transparent**: `Color`

Defined in: [index.d.ts:2526](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2526)

#00000000

***

### Turquoise

> `readonly` `static` **Turquoise**: `Color`

Defined in: [index.d.ts:3050](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3050)

#40E0D0FF

***

### Violet

> `readonly` `static` **Violet**: `Color`

Defined in: [index.d.ts:3054](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3054)

#EE82EEFF

***

### Wheat

> `readonly` `static` **Wheat**: `Color`

Defined in: [index.d.ts:3058](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3058)

#F5DEB3FF

***

### White

> `readonly` `static` **White**: `Color`

Defined in: [index.d.ts:2518](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2518)

#FFFFFFFF

***

### WhiteSmoke

> `readonly` `static` **WhiteSmoke**: `Color`

Defined in: [index.d.ts:3062](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3062)

#F5F5F5FF

***

### Yellow

> `readonly` `static` **Yellow**: `Color`

Defined in: [index.d.ts:3066](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3066)

#FFFF00FF

***

### YellowGreen

> `readonly` `static` **YellowGreen**: `Color`

Defined in: [index.d.ts:3070](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3070)

#9ACD32FF

## Methods

### clone()

> **clone**(): `Color`

Defined in: [index.d.ts:3110](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3110)

Returns a copy of this color.

#### Returns

`Color`

***

### equals()

> **equals**(`other`): `boolean`

Defined in: [index.d.ts:3102](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3102)

Returns `true` if both colors have the same r, g, b, and a values.

```ts
Color.Red.equals(new Color(255, 0, 0)); // true
```

#### Parameters

##### other

`Color`

#### Returns

`boolean`

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:3106](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3106)

Returns a string representation of the color: `"(r, g, b, a)"`.

#### Returns

`string`
