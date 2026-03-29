# Interface: ScheduleOptions


Schedule options for `datetime.waitForSchedule`.

All fields are optional. Missing day fields (`dayOfWeek`, `dayOfMonth`) match
any day. Missing time fields (`hour`, `minute`, `second`) default to `0`.

The method always waits for the **next strictly future** occurrence that
satisfies all specified constraints.

## Properties

### hour?

> `optional` **hour?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target hour (0–23). Defaults to `0`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### minute?

> `optional` **minute?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target minute (0–59). Defaults to `0`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### second?

> `optional` **second?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target second (0–59). Defaults to `0`.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### dayOfWeek?

> `optional` **dayOfWeek?**: [`DayOfWeek`](../enumerations/DayOfWeek.md)

Target weekday. Matches any weekday if omitted.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### dayOfMonth?

> `optional` **dayOfMonth?**: [`number`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number)

Target day of the month (1–31). Matches any day if omitted.
Months shorter than `dayOfMonth` are skipped automatically.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)

***

### signal?

> `optional` **signal?**: [`AbortSignal`](AbortSignal.md)

Abort signal to cancel the wait.

#### Default Value

[`undefined`](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/undefined)
