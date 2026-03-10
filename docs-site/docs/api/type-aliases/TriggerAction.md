# Type Alias: TriggerAction

> **TriggerAction** = [`Macro`](../classes/Macro.md) \| () => [`Macro`](../classes/Macro.md) \| [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void) \| [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<[`Macro`](../classes/Macro.md) \| [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

An action that fires when a trigger is activated: either a [Macro](../classes/Macro.md) to play
directly, or a (possibly async) callback that optionally returns one.

```ts
keyboard.onKey(Key.F5, myMacro);               // play macro directly
keyboard.onKey(Key.F5, () => myMacro);         // callback returning macro
keyboard.onKey(Key.F5, async () => { ... });   // async callback
```
