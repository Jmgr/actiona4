# Interface: Processes

Process listing and inspection.

```ts
const processes = await system.processes.list();
println(processes.length);
```

## Methods

### list()

> **list**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Process`](Process.md)[]\>

Lists all processes

#### Parameters

##### options?

[`ListProcessesOptions`](ListProcessesOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`Process`](Process.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
