# Interface: PlayingSound

A handle to an actively playing sound, allowing control over playback.

```ts
const sound = audio.playFile("music.mp3");
println(sound.duration);  // duration in seconds
sound.volume = 0.5;
sound.playbackRate = 1.5;
sound.pause();
sound.resume();
await sound.finished;  // wait until the sound ends
```

## Properties

### duration?

> `readonly` `optional` **duration**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

The total duration of the sound in seconds, or [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined) if unknown.

***

### finished

> `readonly` **finished**: [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

A promise that resolves when the sound has finished playing.

```ts
const sound = audio.playFile("music.mp3");
await sound.finished;
println("Sound finished!");
```

***

### paused

> `readonly` **paused**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the sound is currently paused.

***

### playbackRate

> **playbackRate**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Sound playing speed

#### Default Value

`1`

***

### volume

> **volume**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Sound volume

#### Default Value

`1`

## Methods

### pause()

> **pause**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Pauses the sound. Use `resume()` to continue playback.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### resume()

> **resume**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Resumes a paused sound.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

***

### stop()

> **stop**(): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Stops the sound permanently.

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)
