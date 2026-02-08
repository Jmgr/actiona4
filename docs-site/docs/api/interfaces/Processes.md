# Interface: Processes

Defined in: [index.d.ts:6239](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6239)

Process listing and inspection.

```ts
const processes = await system.processes.list();
console.log(processes.length);
```

## Methods

### list()

> **list**(`options?`): `Promise`\<readonly [`Process`](Process.md)[]\>

Defined in: [index.d.ts:6243](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6243)

Lists all processes

#### Parameters

##### options?

[`ListProcessesOptions`](ListProcessesOptions.md)

#### Returns

`Promise`\<readonly [`Process`](Process.md)[]\>

***

### toString()

> **toString**(): `string`

Defined in: [index.d.ts:6244](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6244)

#### Returns

`string`
