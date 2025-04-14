# Interface: PressOptions


Options for pressing (and holding) a mouse button.

```ts
// Press the right button at a specific position
mouse.press({ button: Button.Right, position: new Point(100, 200) });

// Press at coordinates using PointLike shorthand
mouse.press({ button: Button.Left, position: {x: 50, y: 100} });
```

## Extended by

- [`ClickOptions`](ClickOptions.md)

## Properties

### button?

> `optional` **button?**: [`Button`](../enumerations/Button.md)

Mouse button to press.

#### Default Value

`Button.Left`

***

### position?

> `optional` **position?**: [`PointLike`](../type-aliases/PointLike.md)

Position to move the cursor to before pressing.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### relativePosition?

> `optional` **relativePosition?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Whether the position is relative to the current cursor position.

#### Default Value

`false`
