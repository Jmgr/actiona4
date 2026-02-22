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

> **playFile**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`PlaySoundOptions`](PlaySoundOptions.md)): [`PlayingSound`](PlayingSound.md)

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

<div class="options-fields">

###### fadeIn?

> `optional` **fadeIn**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Fade in duration

###### Default Value

`0`

***

###### fadeOut?

> `optional` **fadeOut**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Fade out duration

###### Default Value

`0`

***

###### loop?

> `optional` **loop**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound loop

###### Default Value

`false`

***

###### paused?

> `optional` **paused**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound start paused

###### Default Value

`false`

***

###### playbackRate?

> `optional` **playbackRate**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Speed to play the sound at

###### Default Value

`1.0`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the sound playback.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### volume?

> `optional` **volume**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Volume to play the sound at

###### Default Value

`1.0`

</div>

#### Returns

[`PlayingSound`](PlayingSound.md)

***

### playFileAndWait()

> <span class="async-badge">async</span> **playFileAndWait**(`path`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `options?`: [`PlaySoundOptions`](PlaySoundOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

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

<div class="options-fields">

###### fadeIn?

> `optional` **fadeIn**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Fade in duration

###### Default Value

`0`

***

###### fadeOut?

> `optional` **fadeOut**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Fade out duration

###### Default Value

`0`

***

###### loop?

> `optional` **loop**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound loop

###### Default Value

`false`

***

###### paused?

> `optional` **paused**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound start paused

###### Default Value

`false`

***

###### playbackRate?

> `optional` **playbackRate**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Speed to play the sound at

###### Default Value

`1.0`

***

###### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the sound playback.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### volume?

> `optional` **volume**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Volume to play the sound at

###### Default Value

`1.0`

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
