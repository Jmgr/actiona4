# Class: Macro

A recorded macro that can be replayed or saved to disk.

```ts
// Record a macro
const m = await macros.record({ stopKeys: [Key.Escape] });

// Save and reload
await m.save("my_macro.amac");
const loaded = await Macro.load("my_macro.amac");

// Play back
await macros.play(loaded, { speed: 1.5 });
```

## Properties

### eventCount

> `readonly` **eventCount**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns the total number of events in this macro.

***

### duration

> `readonly` **duration**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Returns the total duration of the recording in seconds.

***

### recordedAt

> `readonly` **recordedAt**: [`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

Returns when this macro was recorded.

***

### platform

> `readonly` **platform**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns the platform on which this macro was recorded (`"linux"` or `"windows"`).

## Methods

### load()

> <span class="async-badge">async</span> `static` **load**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Macro`\>

Loads a macro from a gzip-compressed JSON file previously written by `save()`.

```ts
const loaded = await Macro.load("recording.amac");
await macros.play(loaded);
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<`Macro`\>

***

### save()

> <span class="async-badge">async</span> **save**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Saves this macro to a gzip-compressed JSON file.

```ts
await macro.save("recording.amac");
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this macro.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
