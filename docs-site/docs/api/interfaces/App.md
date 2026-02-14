# Interface: App

The global application singleton, providing access to environment information
and execution settings.

```ts
// Get the current version
println(app.version);

// Read environment variables
const home = app.env["HOME"];

// Change working directory
app.setCwd("/tmp");
println(app.cwd);

// Control whether the script waits at the end
app.waitAtEnd = true;
app.waitAtEnd = WaitAtEnd.Automatic;
```

## Properties

### cwd

> `readonly` **cwd**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The current working directory.

```ts
println(app.cwd); // e.g. "/home/user/project"
```

***

### env

> `readonly` **env**: [`Readonly`](https://www.typescriptlang.org/docs/handbook/utility-types.html#readonlytype)\<[`Record`](https://www.typescriptlang.org/docs/handbook/utility-types.html#recordkeys-type)\<[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String), [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String) \| [`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)\>\>

All environment variables as a readonly key-value map.

```ts
const env = app.env;
println(env["HOME"]);
println(env["PATH"]);
```

***

### executablePath

> `readonly` **executablePath**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The path to the running executable.

```ts
println(app.executablePath); // e.g. "/usr/bin/actiona-run"
```

***

### version

> `readonly` **version**: [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

The version of Actiona-cli.

```ts
println(app.version); // e.g. "0.1.0"
```

***

### waitAtEnd

> **waitAtEnd**: [`boolean`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Boolean) \| [`WaitAtEnd`](../enumerations/WaitAtEnd.md)

Should the app wait at the end of execution

## Methods

### setCwd()

> **setCwd**(`cwd`): [`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)

Sets the current working directory.

```ts
app.setCwd("/tmp");
```

#### Parameters

##### cwd

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

#### Returns

[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)
