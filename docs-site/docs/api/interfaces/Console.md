# Interface: Console

Defined in: [index.d.ts:3134](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3134)

The global console singleton for printing output and basic debugging.

```ts
// Print values
console.log("hello", 42, { key: "value" });

// Warnings and errors are styled
console.warn("this is a warning");
console.error("something went wrong");

// Measure elapsed time
console.time("fetch");
// ... do work ...
console.timeEnd("fetch"); // prints "fetch: 1s 234ms - timer ended"

// Count how many times a label is hit
console.count("loop");
console.count("loop");
```

## Methods

### clear()

> **clear**(): `void`

Defined in: [index.d.ts:3162](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3162)

Clears the terminal screen.

#### Returns

`void`

***

### count()

> **count**(`label?`): `void`

Defined in: [index.d.ts:3189](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3189)

Increments and prints a counter for the given label (defaults to `"default"`).

```ts
console.count("loop"); // prints "loop: 1"
console.count("loop"); // prints "loop: 2"
```

#### Parameters

##### label?

`string`

#### Returns

`void`

***

### error()

> **error**(...`args`): `void`

Defined in: [index.d.ts:3158](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3158)

Logs an error in bold red.

#### Parameters

##### args

...`unknown`[]

#### Returns

`void`

***

### info()

> **info**(...`args`): `void`

Defined in: [index.d.ts:3150](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3150)

Logs informational values. Alias for `log`.

#### Parameters

##### args

...`unknown`[]

#### Returns

`void`

***

### log()

> **log**(...`args`): `void`

Defined in: [index.d.ts:3146](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3146)

Logs values to stdout. Alias for `printLn`.

#### Parameters

##### args

...`unknown`[]

#### Returns

`void`

***

### print()

> **print**(...`args`): `void`

Defined in: [index.d.ts:3138](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3138)

Prints values without a trailing newline.

#### Parameters

##### args

...`unknown`[]

#### Returns

`void`

***

### printLn()

> **printLn**(...`args`): `void`

Defined in: [index.d.ts:3142](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3142)

Prints values followed by a newline.

#### Parameters

##### args

...`unknown`[]

#### Returns

`void`

***

### time()

> **time**(`label?`): `void`

Defined in: [index.d.ts:3170](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3170)

Starts a timer with the given label (defaults to `"default"`).

```ts
console.time("myTimer");
```

#### Parameters

##### label?

`string`

#### Returns

`void`

***

### timeEnd()

> **timeEnd**(`label?`): `void`

Defined in: [index.d.ts:3180](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3180)

Stops a timer and prints the elapsed time.

```ts
console.time("myTimer");
// ... do work ...
console.timeEnd("myTimer"); // prints "myTimer: 1s 234ms - timer ended"
```

#### Parameters

##### label?

`string`

#### Returns

`void`

***

### warn()

> **warn**(...`args`): `void`

Defined in: [index.d.ts:3154](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L3154)

Logs a warning in yellow.

#### Parameters

##### args

...`unknown`[]

#### Returns

`void`
