# Interface: TemperatureSensor

A hardware temperature sensor.

```ts
const temperatureSensors = await system.hardware.listTemperatureSensors();
const temperatureSensor = temperatureSensors[0];
if (temperatureSensor) {
  println(temperatureSensor.label, temperatureSensor.temperature);
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
