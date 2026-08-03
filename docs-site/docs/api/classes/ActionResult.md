# Class: ActionResult

Execution control result for Code actions.

Evaluate to one of these values from a Code action script to control which
action runs next. Evaluating to nothing continues with the next sibling
action.

```ts
const result = shouldStop
  ? ActionResult.stop()
  : needsRetry
    ? ActionResult.gotoLabel("retry")
    : ActionResult.branch(ActionBranch.true());

result;
```

## Methods

### nextSibling()

> `static` **nextSibling**(): `ActionResult`

Continues execution with the next sibling action.

#### Returns

`ActionResult`

***

### nextChild()

> `static` **nextChild**(): `ActionResult`

Continues execution with the first child action.

#### Returns

`ActionResult`

***

### branch()

> `static` **branch**(`branch`: [`ActionBranch`](ActionBranch.md)): `ActionResult`

Continues execution with the matching branch.

#### Parameters

##### branch

[`ActionBranch`](ActionBranch.md)

#### Returns

`ActionResult`

***

### gotoLabel()

> `static` **gotoLabel**(`label`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): `ActionResult`

Jumps to the action with the given label.

#### Parameters

##### label

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

`ActionResult`

***

### stop()

> `static` **stop**(): `ActionResult`

Stops action execution.

#### Returns

`ActionResult`

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this action result.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
