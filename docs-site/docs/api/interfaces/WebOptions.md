# Interface: WebOptions

Defined in: [index.d.ts:6670](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6670)

Web request options.

## Properties

### contentType?

> `optional` **contentType**: `string`

Defined in: [index.d.ts:6706](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6706)

Sets the content-type header.
Overrides any content-type set by other fields.

#### Default Value

`undefined`

***

### form?

> `optional` **form**: `Record`\<`string`, `string` \| `undefined`\>

Defined in: [index.d.ts:6712](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6712)

Form data as strings.
Sets content-type to "application/x-www-form-urlencoded".

#### Default Value

`undefined`

***

### headers?

> `optional` **headers**: `Record`\<`string`, `string` \| `undefined`\>

Defined in: [index.d.ts:6690](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6690)

Additional HTTP headers to send with the request.

#### Default Value

`undefined`

***

### method?

> `optional` **method**: [`Method`](../enumerations/Method.md)

Defined in: [index.d.ts:6695](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6695)

HTTP method to use for the request.

#### Default Value

`Method.Get`

***

### multipart?

> `optional` **multipart**: [`MultipartForm`](../classes/MultipartForm.md)

Defined in: [index.d.ts:6723](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6723)

Form multipart data.
Sets content-type and content-length appropriately.

#### Default Value

`undefined`

***

### password?

> `optional` **password**: `string`

Defined in: [index.d.ts:6685](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6685)

Password for HTTP basic authentication.

#### Default Value

`undefined`

***

### query?

> `optional` **query**: `Record`\<`string`, `string` \| `undefined`\>

Defined in: [index.d.ts:6717](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6717)

Additional query parameters.

#### Default Value

`undefined`

***

### signal?

> `optional` **signal**: [`AbortSignal`](AbortSignal.md)

Defined in: [index.d.ts:6675](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6675)

Abort signal to cancel the request.

#### Default Value

`undefined`

***

### timeout?

> `optional` **timeout**: `number`

Defined in: [index.d.ts:6700](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6700)

Request timeout duration.

#### Default Value

`undefined`

***

### userName?

> `optional` **userName**: `string`

Defined in: [index.d.ts:6680](https://github.com/Jmgr/actiona-ng/blob/f1176bbc3f17a88f0c5c87b23e11adcc98b5adb1/tests/src/index.d.ts#L6680)

User name for HTTP basic authentication.

#### Default Value

`undefined`
