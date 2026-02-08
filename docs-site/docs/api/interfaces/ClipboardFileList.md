# Interface: ClipboardFileList

Defined in: [index.d.ts:2432](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2432)

Provides file list clipboard operations.

```ts
await clipboard.fileList.set(["/home/user/doc.pdf", "/home/user/img.png"]);
const files = await clipboard.fileList.get();
```

## Methods

### get()

> **get**(`mode?`): `Promise`\<readonly `string`[]\>

Defined in: [index.d.ts:2440](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2440)

Gets the clipboard file list content.

#### Parameters

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<readonly `string`[]\>

***

### set()

> **set**(`fileList`, `mode?`): `Promise`\<`void`\>

Defined in: [index.d.ts:2436](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L2436)

Sets the clipboard file list content.

#### Parameters

##### fileList

`string`[]

##### mode?

[`ClipboardMode`](../enumerations/ClipboardMode.md)

#### Returns

`Promise`\<`void`\>
