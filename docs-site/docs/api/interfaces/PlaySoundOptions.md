# Interface: PlaySoundOptions


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

### volume?

> `optional` **volume?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Volume to play the sound at

#### Default Value

`1`

***

### playbackRate?

> `optional` **playbackRate?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Speed to play the sound at

#### Default Value

`1`

***

### paused?

> `optional` **paused?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound start paused

#### Default Value

`false`

***

### loop?

> `optional` **loop?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Should the sound loop

#### Default Value

`false`

***

### fadeIn?

> `optional` **fadeIn?**: [`DurationLike`](../type-aliases/DurationLike.md)

Fade in duration

#### Default Value

`0`

***

### fadeOut?

> `optional` **fadeOut?**: [`DurationLike`](../type-aliases/DurationLike.md)

Fade out duration

#### Default Value

`0`

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the sound playback.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
