# Interface: Processes

Process listing and inspection.

```ts
const processes = await system.processes.list();
println(processes.length);
```

## Methods

### list()

> <span class="async-badge">async</span> **list**(`options?`: [`ListProcessesOptions`](ListProcessesOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

Lists all processes

#### Parameters

##### options?

[`ListProcessesOptions`](ListProcessesOptions.md)

<div class="options-fields">

###### rescan?

> `optional` **rescan?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Rescan

###### Default Value

`true`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

***

### find()

> <span class="async-badge">async</span> **find**(`options`: [`ProcessesFindOptions`](ProcessesFindOptions.md)): [`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

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

<div class="options-fields">

###### pid?

> `optional` **pid?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by process ID.
When undefined, any PID is accepted.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### parentPid?

> `optional` **parentPid?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Match by parent process ID.
When undefined, parent PID is not filtered.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### name?

> `optional` **name?**: [`NameLike`](../type-aliases/NameLike.md)

Match by process name.
When undefined, name is not filtered.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### status?

> `optional` **status?**: [`ProcessStatus`](../enumerations/ProcessStatus.md)

Match by process status.
When undefined, status is not filtered.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### rescan?

> `optional` **rescan?**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean)

Refresh process list before filtering.

###### Default Value

`true`

</div>

#### Returns

[`Promise`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Promise)\<readonly [`ProcessInfo`](ProcessInfo.md)[]\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of this process list.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
