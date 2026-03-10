# Class: MessageBoxButtons

Button configurations for message boxes.

Use the static factory methods to create button sets.

```ts
const buttons = MessageBoxButtons.ok();
const buttons2 = MessageBoxButtons.yesNoCancel();
const buttons3 = MessageBoxButtons.okCancelCustom("Save", "Discard");
```

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this set of message box buttons.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

### ok()

> `static` **ok**(): `MessageBoxButtons`

Creates an OK button.

#### Returns

`MessageBoxButtons`

***

### okCancel()

> `static` **okCancel**(): `MessageBoxButtons`

Creates OK and Cancel buttons.

#### Returns

`MessageBoxButtons`

***

### okCancelCustom()

> `static` **okCancelCustom**(`okLabel`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `cancelLabel`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): `MessageBoxButtons`

Creates OK and Cancel buttons with custom labels.

#### Parameters

##### okLabel

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### cancelLabel

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

`MessageBoxButtons`

***

### okCustom()

> `static` **okCustom**(`okLabel`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): `MessageBoxButtons`

Creates an OK button with a custom label.

#### Parameters

##### okLabel

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

`MessageBoxButtons`

***

### yesNo()

> `static` **yesNo**(): `MessageBoxButtons`

Creates Yes and No buttons.

#### Returns

`MessageBoxButtons`

***

### yesNoCancel()

> `static` **yesNoCancel**(): `MessageBoxButtons`

Creates Yes, No, and Cancel buttons.

#### Returns

`MessageBoxButtons`

***

### yesNoCancelCustom()

> `static` **yesNoCancelCustom**(`yesLabel`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `noLabel`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), `cancelLabel`: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)): `MessageBoxButtons`

Creates Yes, No, and Cancel buttons with custom labels.

#### Parameters

##### yesLabel

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### noLabel

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

##### cancelLabel

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

`MessageBoxButtons`
