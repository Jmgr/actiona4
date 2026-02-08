# Interface: Component

Defined in: [index.d.ts:5801](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5801)

A hardware component (for example a thermal sensor).

```ts
const components = await system.hardware.listComponents();
const component = components[0];
if (component) {
console.log(component.label, component.temperature);
}
```

## Properties

### criticalTemperature?

> `readonly` `optional` **criticalTemperature**: `number`

Defined in: [index.d.ts:5821](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5821)

Critical temperature

***

### id?

> `readonly` `optional` **id**: `string`

Defined in: [index.d.ts:5809](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5809)

ID

***

### label

> `readonly` **label**: `string`

Defined in: [index.d.ts:5805](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5805)

Label

***

### maxTemperature?

> `readonly` `optional` **maxTemperature**: `number`

Defined in: [index.d.ts:5817](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5817)

Maximum temperature

***

### temperature?

> `readonly` `optional` **temperature**: `number`

Defined in: [index.d.ts:5813](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5813)

Temperature

## Methods

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:5822](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L5822)

#### Returns

`string`
