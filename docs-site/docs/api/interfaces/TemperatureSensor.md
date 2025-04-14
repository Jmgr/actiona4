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

### label

> `readonly` **label**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Label

***

### id?

> `readonly` `optional` **id?**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

ID

***

### temperature?

> `readonly` `optional` **temperature?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Temperature

***

### maxTemperature?

> `readonly` `optional` **maxTemperature?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Maximum temperature

***

### criticalTemperature?

> `readonly` `optional` **criticalTemperature?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Critical temperature

## Methods

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this temperature sensor.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
