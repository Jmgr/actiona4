# Class: ActionBranch

Branch target used by `ActionResult.branch`.

```ts
ActionResult.branch(ActionBranch.yes());
ActionResult.branch(ActionBranch.custom("retry"));
```

## Methods

### yes()

> `static` **yes**(): `ActionBranch`

Selects the `yes` branch.

#### Returns

`ActionBranch`

***

### no()

> `static` **no**(): `ActionBranch`

Selects the `no` branch.

#### Returns

`ActionBranch`

***

### cancel()

> `static` **cancel**(): `ActionBranch`

Selects the `cancel` branch.

#### Returns

`ActionBranch`

***

### true()

> `static` **true**(): `ActionBranch`

Selects the `true` branch.

#### Returns

`ActionBranch`

***

### false()

> `static` **false**(): `ActionBranch`

Selects the `false` branch.

#### Returns

`ActionBranch`

***

### custom()

> `static` **custom**(`name`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): `ActionBranch`

Selects a custom named branch.

#### Parameters

##### name

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

`ActionBranch`

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this action branch.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
