# Interface: Component

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

> `readonly` `optional` **criticalTemperature**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Critical temperature

***

### id?

> `readonly` `optional` **id**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

ID

***

### label

> `readonly` **label**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label

***

### maxTemperature?

> `readonly` `optional` **maxTemperature**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Maximum temperature

***

### temperature?

> `readonly` `optional` **temperature**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Temperature

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
