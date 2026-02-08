# Interface: Audio

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

Plays a sound file and returns a `PlayingSound` handle for controlling playback.

```ts
const sound = audio.playFile("music.mp3");
sound.volume = 0.5;
```

#### Parameters

##### path

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`PlaySoundOptions`](PlaySoundOptions.md)

#### Returns

[`PlayingSound`](PlayingSound.md)

***

### playFileAndWait()

> **playFileAndWait**(`path`, `options?`): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### options?

[`PlaySoundOptions`](PlaySoundOptions.md)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>
