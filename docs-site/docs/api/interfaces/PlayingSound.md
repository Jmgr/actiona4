# Interface: PlayingSound

Defined in: [index.d.ts:2280](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2280)

A handle to an actively playing sound, allowing control over playback.

```ts
const sound = audio.playFile("music.mp3");
console.log(sound.duration);  // duration in seconds
sound.volume = 0.5;
sound.playbackRate = 1.5;
sound.pause();
sound.resume();
await sound.finished;  // wait until the sound ends
```

## Properties

### duration?

> `readonly` `optional` **duration**: `number`

Defined in: [index.d.ts:2298](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2298)

The total duration of the sound in seconds, or `undefined` if unknown.

***

### finished

> `readonly` **finished**: `Promise`\<`void`\>

Defined in: [index.d.ts:2308](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2308)

A promise that resolves when the sound has finished playing.

```ts
const sound = audio.playFile("music.mp3");
await sound.finished;
console.log("Sound finished!");
```

***

### paused

> `readonly` **paused**: `boolean`

Defined in: [index.d.ts:2294](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2294)

Whether the sound is currently paused.

***

### playbackRate

> **playbackRate**: `number`

Defined in: [index.d.ts:2290](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2290)

Sound playing speed

#### Default Value

`1`

***

### volume

> **volume**: `number`

Defined in: [index.d.ts:2285](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2285)

Sound volume

#### Default Value

`1`

## Methods

### pause()

> **pause**(): `void`

Defined in: [index.d.ts:2312](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2312)

Pauses the sound. Use `resume()` to continue playback.

#### Returns

`void`

***

### resume()

> **resume**(): `void`

Defined in: [index.d.ts:2316](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2316)

Resumes a paused sound.

#### Returns

`void`

***

### stop()

> **stop**(): `void`

Defined in: [index.d.ts:2320](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2320)

Stops the sound permanently.

#### Returns

`void`
