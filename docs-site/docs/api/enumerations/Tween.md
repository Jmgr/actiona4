# Enumeration: Tween

Defined in: [index.d.ts:1891](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1891)

Tweening functions for smooth mouse movement.

```ts
// Move with a bounce effect
await mouse.move(500, 300, { tween: Tween.BounceOut });

// Move with linear interpolation (no easing)
await mouse.move(100, 100, { tween: Tween.Linear });
```

## Enumeration Members

### BackIn

> **BackIn**: `number`

Defined in: [index.d.ts:1895](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1895)

Starts slowly, then accelerates with an overshoot.

***

### BackInOut

> **BackInOut**: `number`

Defined in: [index.d.ts:1900](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1900)

Starts and ends with an overshoot, accelerating in between.

***

### BackOut

> **BackOut**: `number`

Defined in: [index.d.ts:1905](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1905)

Starts quickly, then decelerates with an overshoot.

***

### BounceIn

> **BounceIn**: `number`

Defined in: [index.d.ts:1910](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1910)

Starts by bouncing off the start point.

***

### BounceInOut

> **BounceInOut**: `number`

Defined in: [index.d.ts:1915](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1915)

Bounces at both the start and end points.

***

### BounceOut

> **BounceOut**: `number`

Defined in: [index.d.ts:1920](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1920)

Ends with a bounce effect.

***

### CircIn

> **CircIn**: `number`

Defined in: [index.d.ts:1925](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1925)

Starts slowly and accelerates in a circular motion.

***

### CircInOut

> **CircInOut**: `number`

Defined in: [index.d.ts:1930](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1930)

Starts and ends slowly with a circular motion.

***

### CircOut

> **CircOut**: `number`

Defined in: [index.d.ts:1935](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1935)

Ends slowly with a circular motion.

***

### CubicIn

> **CubicIn**: `number`

Defined in: [index.d.ts:1940](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1940)

Starts slowly and accelerates cubically.

***

### CubicInOut

> **CubicInOut**: `number`

Defined in: [index.d.ts:1945](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1945)

Starts and ends slowly with a cubic acceleration.

***

### CubicOut

> **CubicOut**: `number`

Defined in: [index.d.ts:1950](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1950)

Ends slowly with a cubic deceleration.

***

### ElasticIn

> **ElasticIn**: `number`

Defined in: [index.d.ts:1955](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1955)

Starts with an elastic effect, overshooting the target.

***

### ElasticInOut

> **ElasticInOut**: `number`

Defined in: [index.d.ts:1960](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1960)

Starts and ends with an elastic effect.

***

### ElasticOut

> **ElasticOut**: `number`

Defined in: [index.d.ts:1965](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1965)

Ends with an elastic effect, overshooting the target.

***

### ExpoIn

> **ExpoIn**: `number`

Defined in: [index.d.ts:1970](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1970)

Starts slowly and accelerates exponentially.

***

### ExpoInOut

> **ExpoInOut**: `number`

Defined in: [index.d.ts:1975](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1975)

Starts and ends slowly with an exponential acceleration.

***

### ExpoOut

> **ExpoOut**: `number`

Defined in: [index.d.ts:1980](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1980)

Ends slowly with an exponential deceleration.

***

### Linear

> **Linear**: `number`

Defined in: [index.d.ts:1985](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1985)

A linear tween with no acceleration or deceleration.

***

### QuadIn

> **QuadIn**: `number`

Defined in: [index.d.ts:1990](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1990)

Starts slowly and accelerates quadratically.

***

### QuadInOut

> **QuadInOut**: `number`

Defined in: [index.d.ts:1995](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L1995)

Starts and ends slowly with a quadratic acceleration.

***

### QuadOut

> **QuadOut**: `number`

Defined in: [index.d.ts:2000](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2000)

Ends slowly with a quadratic deceleration.

***

### QuartIn

> **QuartIn**: `number`

Defined in: [index.d.ts:2005](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2005)

Starts slowly and accelerates quartically.

***

### QuartInOut

> **QuartInOut**: `number`

Defined in: [index.d.ts:2010](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2010)

Starts and ends slowly with a quartic acceleration.

***

### QuartOut

> **QuartOut**: `number`

Defined in: [index.d.ts:2015](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2015)

Ends slowly with a quartic deceleration.

***

### QuintIn

> **QuintIn**: `number`

Defined in: [index.d.ts:2020](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2020)

Starts slowly and accelerates quintically.

***

### QuintInOut

> **QuintInOut**: `number`

Defined in: [index.d.ts:2025](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2025)

Starts and ends slowly with a quintic acceleration.

***

### QuintOut

> **QuintOut**: `number`

Defined in: [index.d.ts:2030](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2030)

Ends slowly with a quintic deceleration.

***

### SineIn

> **SineIn**: `number`

Defined in: [index.d.ts:2035](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2035)

Starts slowly and accelerates sinusoidally.

***

### SineInOut

> **SineInOut**: `number`

Defined in: [index.d.ts:2040](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2040)

Starts and ends slowly with a sinusoidal acceleration.

***

### SineOut

> **SineOut**: `number`

Defined in: [index.d.ts:2045](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2045)

Ends slowly with a sinusoidal deceleration.
