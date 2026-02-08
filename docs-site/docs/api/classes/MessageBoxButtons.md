# Class: MessageBoxButtons

Defined in: [index.d.ts:6590](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6590)

Button configurations for message boxes.

Use the static factory methods to create button sets.

```ts
const buttons = MessageBoxButtons.ok();
const buttons2 = MessageBoxButtons.yesNoCancel();
const buttons3 = MessageBoxButtons.okCancelCustom("Save", "Discard");
```

## Methods

### ok()

> `static` **ok**(): `MessageBoxButtons`

Defined in: [index.d.ts:6595](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6595)

Creates an OK button.

#### Returns

`MessageBoxButtons`

***

### okCancel()

> `static` **okCancel**(): `MessageBoxButtons`

Defined in: [index.d.ts:6603](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6603)

Creates OK and Cancel buttons.

#### Returns

`MessageBoxButtons`

***

### okCancelCustom()

> `static` **okCancelCustom**(`okLabel`, `cancelLabel`): `MessageBoxButtons`

Defined in: [index.d.ts:6607](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6607)

Creates OK and Cancel buttons with custom labels.

#### Parameters

##### okLabel

`string`

##### cancelLabel

`string`

#### Returns

`MessageBoxButtons`

***

### okCustom()

> `static` **okCustom**(`okLabel`): `MessageBoxButtons`

Defined in: [index.d.ts:6599](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6599)

Creates an OK button with a custom label.

#### Parameters

##### okLabel

`string`

#### Returns

`MessageBoxButtons`

***

### yesNo()

> `static` **yesNo**(): `MessageBoxButtons`

Defined in: [index.d.ts:6611](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6611)

Creates Yes and No buttons.

#### Returns

`MessageBoxButtons`

***

### yesNoCancel()

> `static` **yesNoCancel**(): `MessageBoxButtons`

Defined in: [index.d.ts:6615](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6615)

Creates Yes, No, and Cancel buttons.

#### Returns

`MessageBoxButtons`

***

### yesNoCancelCustom()

> `static` **yesNoCancelCustom**(`yesLabel`, `noLabel`, `cancelLabel`): `MessageBoxButtons`

Defined in: [index.d.ts:6619](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6619)

Creates Yes, No, and Cancel buttons with custom labels.

#### Parameters

##### yesLabel

`string`

##### noLabel

`string`

##### cancelLabel

`string`

#### Returns

`MessageBoxButtons`
