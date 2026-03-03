# Interface: DragOptions


Options for drag and drop operations.

```ts
await mouse.dragAndDrop({ x: 100, y: 100 }, { x: 500, y: 500 }, {
  speed: 500,
  tween: Tween.Linear,
});
```

## Extends

- `MoveOptions`

## Properties

### button?

> `optional` **button**: [`Button`](../enumerations/Button.md)

Mouse button to use for dragging.

#### Default Value

`Button.Left`
