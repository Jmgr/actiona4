# Interface: Processes

Process listing and inspection.

```ts
const processes = await system.processes.list();
println(processes.length);
```

## Methods

### find()

> **find**(`options`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

Finds processes matching the provided criteria.
```ts
const byPid = await system.processes.find({ pid: 12345 });
const byParent = await system.processes.find({ parentPid: 1 });
const byName = await system.processes.find({ name: new Wildcard("my-app*") });
const running = await system.processes.find({ status: ProcessStatus.Run });
const exact = await system.processes.find({ pid: 12345, name: "my-app" });
```

#### Parameters

##### options

[`ProcessesFindOptions`](ProcessesFindOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

***

### list()

> **list**(`options?`): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

Lists all processes

#### Parameters

##### options?

[`ListProcessesOptions`](ListProcessesOptions.md)

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
