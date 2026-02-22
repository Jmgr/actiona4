# Interface: PlaySoundOptions

**`Expand`**

Options for playing a sound file.

```ts
// Play with default options
audio.playFile("sound.wav");

// Play at half volume, looping, with a fade in
audio.playFile("music.mp3", {
    volume: 0.5,
    loop: true,
    fadeIn: 2000,
});
```

## Properties

### fadeIn?

> `optional` **fadeIn**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Fade in duration

#### Default Value

`0`

***

### fadeOut?

> `optional` **fadeOut**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Fade out duration

#### Default Value

`0`

***

### loop?

> `optional` **loop**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound loop

#### Default Value

`false`

***

### paused?

> `optional` **paused**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound start paused

#### Default Value

`false`

***

### playbackRate?

> `optional` **playbackRate**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Speed to play the sound at

#### Default Value

`1.0`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the sound playback.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### volume?

> `optional` **volume**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Volume to play the sound at

#### Default Value

`1.0`
