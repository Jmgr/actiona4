# Interface: App

Defined in: [index.d.ts:2113](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2113)

The global application singleton, providing access to environment information
and execution settings.

```ts
// Get the current version
console.log(app.version);

// Read environment variables
const home = app.env["HOME"];

// Change working directory
app.setCwd("/tmp");
console.log(app.cwd);

// Control whether the script waits at the end
app.waitAtEnd = true;
app.waitAtEnd = WaitAtEnd.Automatic;
```

## Properties

### cwd

> `readonly` **cwd**: `string`

Defined in: [index.d.ts:2143](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2143)

The current working directory.

```ts
console.log(app.cwd); // e.g. "/home/user/project"
```

***

### env

> `readonly` **env**: `Readonly`\<`Record`\<`string`, `string` \| `undefined`\>\>

Defined in: [index.d.ts:2135](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2135)

All environment variables as a readonly key-value map.

```ts
const env = app.env;
console.log(env["HOME"]);
console.log(env["PATH"]);
```

***

### executablePath

> `readonly` **executablePath**: `string`

Defined in: [index.d.ts:2151](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2151)

The path to the running executable.

```ts
console.log(app.executablePath); // e.g. "/usr/bin/actiona-ng-cli"
```

***

### version

> `readonly` **version**: `string`

Defined in: [index.d.ts:2125](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2125)

The version of Actiona-cli.

```ts
console.log(app.version); // e.g. "0.1.0"
```

***

### waitAtEnd

> **waitAtEnd**: `boolean` \| [`WaitAtEnd`](../enumerations/WaitAtEnd.md)

Defined in: [index.d.ts:2117](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2117)

Should the app wait at the end of execution

## Methods

### setCwd()

> **setCwd**(`cwd`): `void`

Defined in: [index.d.ts:2159](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2159)

Sets the current working directory.

```ts
app.setCwd("/tmp");
```

#### Parameters

##### cwd

`string`

#### Returns

`void`
