# Function: formatPercent()

> **formatPercent**(`percent`, `precision?`): `string`

Defined in: [index.d.ts:79](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L79)

Formats a percentage value and appends `%`.

```ts
formatPercent(50);          // "50%"
formatPercent(50.005);      // "50.01%"
formatPercent(12.3456, 1);  // "12.3%"
```

## Parameters

### percent

`number`

### precision?

`number`

## Returns

`string`
