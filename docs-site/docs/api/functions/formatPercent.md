# Function: formatPercent()

> **formatPercent**(`percent`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `precision?`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Formats a percentage value and appends `%`.

```ts
formatPercent(50);          // "50%"
formatPercent(50.005);      // "50.01%"
formatPercent(12.3456, 1);  // "12.3%"
```

## Parameters

### percent

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

### precision?

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

## Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
