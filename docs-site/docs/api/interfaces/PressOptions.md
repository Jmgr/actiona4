# Interface: PressOptions

Defined in: [index.d.ts:7125](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7125)

Options for pressing (and holding) a mouse button.

```ts
// Press the right button at a specific position
await mouse.press({ button: Button.Right, position: new Point(100, 200) });

// Press at coordinates using PointLike shorthand
await mouse.press({ button: Button.Left, position: {x: 50, y: 100} });
```

## Extended by

- [`ClickOptions`](ClickOptions.md)

## Properties

### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

Defined in: [index.d.ts:7130](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7130)

Mouse button to press.

#### Default Value

`Button.Left`

***

### position?

> `optional` **position**: [`Point`](../classes/Point.md)

Defined in: [index.d.ts:7135](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7135)

Position to move the cursor to before pressing.

#### Default Value

`undefined`

***

### relativePosition?

> `optional` **relativePosition**: `boolean`

Defined in: [index.d.ts:7140](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L7140)

Whether the position is relative to the current cursor position.

#### Default Value

`false`
