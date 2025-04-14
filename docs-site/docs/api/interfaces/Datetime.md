# Interface: Datetime

Provides time-condition based waiting.

All `waitFor*` methods return a cancellable `Task` that resolves at the
next occurrence of the specified time condition.

```ts
// Wait until next 13:15
await datetime.waitForSchedule({ hour: 13, minute: 15 });
```

```ts
// Wait until next Monday at 09:30
await datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday, hour: 9, minute: 30 });
```

```ts
// Wait for a duration (alias for sleep)
await datetime.waitFor("2s");
```

```ts
// Wait until a specific date
await datetime.waitUntil(new Date("2026-12-31T23:59:59"));
```

## Methods

### waitFor()

> <span class="async-badge">async</span> **waitFor**(`duration`: [`DurationLike`](../type-aliases/DurationLike.md), `options?`: [`WaitOptions`](WaitOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits for the given duration. Alias for `sleep`.

```ts
await datetime.waitFor(500);     // 500 ms
await datetime.waitFor("1s");    // 1 second
await datetime.waitFor("30m");   // 30 minutes
```

Numeric values are interpreted as milliseconds.

#### Parameters

##### duration

[`DurationLike`](../type-aliases/DurationLike.md)

##### options?

[`WaitOptions`](WaitOptions.md)

<div class="options-fields">

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### waitUntil()

> <span class="async-badge">async</span> **waitUntil**(`date`: [`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until a specific `Date` is reached.

Resolves immediately if the date is in the past.

```ts
await datetime.waitUntil(new Date("2026-12-31T23:59:59"));
```

```ts
// Wait until 1 second from now
const target = new Date(Date.now() + 1000);
await datetime.waitUntil(target);
```

#### Parameters

##### date

[`Date`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Date)

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### waitForSchedule()

> <span class="async-badge">async</span> **waitForSchedule**(`options`: [`ScheduleOptions`](ScheduleOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until the next occurrence matching the given schedule.

Missing day fields (`dayOfWeek`, `dayOfMonth`) match any day.
Missing time fields (`hour`, `minute`) default to `0`.
Always waits for the *next strictly future* occurrence.

```ts
// Wait until next 13:15 (any day)
await datetime.waitForSchedule({ hour: 13, minute: 15 });
```

```ts
// Wait until next Monday at 09:30
await datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday, hour: 9, minute: 30 });
```

```ts
// Wait until the next :15 of any hour
await datetime.waitForSchedule({ minute: 15 });
```

```ts
// Wait until the 1st of every month at midnight
while (true) {
  await datetime.waitForSchedule({ dayOfMonth: 1 });
  doMonthlyTask();
}
```

```ts
// With cancellation
const controller = new AbortController();
await datetime.waitForSchedule({ hour: 9, signal: controller.signal });
```

#### Parameters

##### options

[`ScheduleOptions`](ScheduleOptions.md)

<div class="options-fields">

###### hour?

> `optional` **hour?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target hour (0–23). Defaults to `0`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### minute?

> `optional` **minute?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target minute (0–59). Defaults to `0`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### second?

> `optional` **second?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target second (0–59). Defaults to `0`.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### dayOfWeek?

> `optional` **dayOfWeek?**: [`DayOfWeek`](../enumerations/DayOfWeek.md)

<div class="options-fields">

###### Monday

> **Monday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Monday`

***

###### Tuesday

> **Tuesday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Tuesday`

***

###### Wednesday

> **Wednesday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Wednesday`

***

###### Thursday

> **Thursday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Thursday`

***

###### Friday

> **Friday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Friday`

***

###### Saturday

> **Saturday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Saturday`

***

###### Sunday

> **Sunday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Sunday`

</div>

Target weekday. Matches any weekday if omitted.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### dayOfMonth?

> `optional` **dayOfMonth?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target day of the month (1–31). Matches any day if omitted.
Months shorter than `dayOfMonth` are skipped automatically.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

###### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

###### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

</div>

#### Returns

[`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

***

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `datetime` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)
