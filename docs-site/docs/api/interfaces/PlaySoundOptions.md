# Interface: PlaySoundOptions

Defined in: [index.d.ts:2181](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2181)

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

> `optional` **fadeIn**: `number`

Defined in: [index.d.ts:2206](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2206)

Fade in duration

#### Default Value

`0`

***

### fadeOut?

> `optional` **fadeOut**: `number`

Defined in: [index.d.ts:2211](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2211)

Fade out duration

#### Default Value

`0`

***

### loop?

> `optional` **loop**: `boolean`

Defined in: [index.d.ts:2201](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2201)

Should the sound loop

#### Default Value

`false`

***

### paused?

> `optional` **paused**: `boolean`

Defined in: [index.d.ts:2196](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2196)

Should the sound start paused

#### Default Value

`false`

***

### playbackRate?

> `optional` **playbackRate**: `number`

Defined in: [index.d.ts:2191](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2191)

Speed to play the sound at

#### Default Value

`1.0`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:2216](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2216)

Abort signal to cancel the sound playback.

#### Default Value

`undefined`

***

### volume?

> `optional` **volume**: `number`

Defined in: [index.d.ts:2186](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2186)

Volume to play the sound at

#### Default Value

`1.0`
