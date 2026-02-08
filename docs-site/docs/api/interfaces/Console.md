# Interface: Console

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

> **clear**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Clears the terminal screen.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### count()

> **count**(`label?`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Increments and prints a counter for the given label (defaults to `"default"`).

```ts
console.count("loop"); // prints "loop: 1"
console.count("loop"); // prints "loop: 2"
```

#### Parameters

##### label?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### error()

> **error**(...`args`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Logs an error in bold red.

#### Parameters

##### args

...`unknown`[]

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### info()

> **info**(...`args`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Logs informational values. Alias for `log`.

#### Parameters

##### args

...`unknown`[]

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### log()

> **log**(...`args`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Logs values to stdout. Alias for `printLn`.

#### Parameters

##### args

...`unknown`[]

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### print()

> **print**(...`args`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Prints values without a trailing newline.

#### Parameters

##### args

...`unknown`[]

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### printLn()

> **printLn**(...`args`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Prints values followed by a newline.

#### Parameters

##### args

...`unknown`[]

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### time()

> **time**(`label?`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Starts a timer with the given label (defaults to `"default"`).

```ts
console.time("myTimer");
```

#### Parameters

##### label?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### timeEnd()

> **timeEnd**(`label?`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Stops a timer and prints the elapsed time.

```ts
console.time("myTimer");
// ... do work ...
console.timeEnd("myTimer"); // prints "myTimer: 1s 234ms - timer ended"
```

#### Parameters

##### label?

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### warn()

> **warn**(...`args`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Logs a warning in yellow.

#### Parameters

##### args

...`unknown`[]

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)
