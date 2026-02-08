# Interface: Audio

Defined in: [index.d.ts:2236](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2236)

The global audio singleton for playing sound files.

```ts
// Play a sound and forget about it
audio.playFile("notification.wav");

// Play a sound and wait for it to finish
await audio.playFileAndWait("alert.wav");

// Play with options and control playback
const sound = audio.playFile("music.mp3", { volume: 0.8, loop: true });
sound.pause();
sound.resume();
sound.stop();
```

## Methods

### playFile()

> **playFile**(`path`, `options?`): [`PlayingSound`](PlayingSound.md)

Defined in: [index.d.ts:2245](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2245)

Plays a sound file and returns a `PlayingSound` handle for controlling playback.

```ts
const sound = audio.playFile("music.mp3");
sound.volume = 0.5;
```

#### Parameters

##### path

`string`

##### options?

[`PlaySoundOptions`](PlaySoundOptions.md)

#### Returns

[`PlayingSound`](PlayingSound.md)

***

### playFileAndWait()

> **playFileAndWait**(`path`, `options?`): [`Task`](../type-aliases/Task.md)\<`void`\>

Defined in: [index.d.ts:2260](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2260)

Plays a sound file and waits for it to finish.

```ts
await audio.playFileAndWait("alert.wav");

// With a fade out and abort signal
const controller = new AbortController();
await audio.playFileAndWait("long-track.mp3", {
fadeOut: 1000,
signal: controller.signal,
});
```

#### Parameters

##### path

`string`

##### options?

[`PlaySoundOptions`](PlaySoundOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<`void`\>
