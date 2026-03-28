# Interface: Datetime

Provides time-condition based waiting.

All `waitFor*` methods return a cancellable `Task` that resolves at the
next occurrence of the specified time condition.

```ts
// Wait until next 13:00:00
await datetime.waitForHour(13);
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

### toString()

> **toString**(): [`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

Returns a string representation of the `datetime` singleton.

#### Returns

[`string`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/String)

***

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

### waitForDayOfMonth()

> <span class="async-badge">async</span> **waitForDayOfMonth**(`day`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`WaitOptions`](WaitOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until the next occurrence of the given day of the month (1–31) at midnight.

Always waits for the *next* occurrence: if the current day of month is
already past (or equal to) `day`, it waits until that day in the next month.
Months that are shorter than `day` are skipped automatically.

```ts
// Run something on the 1st of every month
while (true) {
  await datetime.waitForDayOfMonth(1);
  doMonthlyTask();
}
```

#### Parameters

##### day

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

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

### waitForDayOfWeek()

> <span class="async-badge">async</span> **waitForDayOfWeek**(`day`: [`DayOfWeek`](../enumerations/DayOfWeek.md), `options?`: [`WaitOptions`](WaitOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until the next occurrence of the given weekday at midnight.

Always waits for the *next* occurrence: if today is already that weekday,
it waits until the same weekday next week.

```ts
// Run something every Monday
while (true) {
  await datetime.waitForDayOfWeek(DayOfWeek.Monday);
  doWeeklyTask();
}
```

```ts
// With cancellation
const controller = new AbortController();
await datetime.waitForDayOfWeek(DayOfWeek.Friday, { signal: controller.signal });
```

#### Parameters

##### day

[`DayOfWeek`](../enumerations/DayOfWeek.md)

<div class="options-fields">

###### Friday

> **Friday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Friday`

***

###### Monday

> **Monday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Monday`

***

###### Saturday

> **Saturday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Saturday`

***

###### Sunday

> **Sunday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Sunday`

***

###### Thursday

> **Thursday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Thursday`

***

###### Tuesday

> **Tuesday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Tuesday`

***

###### Wednesday

> **Wednesday**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

`DayOfWeek.Wednesday`

</div>

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

### waitForHour()

> <span class="async-badge">async</span> **waitForHour**(`hour`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`WaitOptions`](WaitOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until the next occurrence of the given hour (0–23) at minute 0, second 0.

Always waits for the *next* occurrence: if the current time is already past
`hour:00:00` today, it waits until tomorrow.

```ts
// Run something every day at 09:00
while (true) {
  await datetime.waitForHour(9);
  doMorningTask();
}
```

```ts
// With cancellation support
const controller = new AbortController();
await datetime.waitForHour(13, { signal: controller.signal });
```

#### Parameters

##### hour

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

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

### waitForMinute()

> <span class="async-badge">async</span> **waitForMinute**(`minute`: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number), `options?`: [`WaitOptions`](WaitOptions.md)): [`Task`](../type-aliases/Task.md)\<[`void`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Operators/void)\>

Waits until the next occurrence of the given minute (0–59), at second 0.

Always waits for the *next* occurrence: if the current minute is already
past `minute:00`, it waits until the same minute in the next hour.

```ts
// Run something every hour at HH:30:00
while (true) {
  await datetime.waitForMinute(30);
  doHalfHourTask();
}
```

#### Parameters

##### minute

[`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

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
