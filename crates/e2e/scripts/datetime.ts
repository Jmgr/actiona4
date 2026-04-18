const waitStart = Date.now();
await datetime.waitFor("100ms");
assert(Date.now() - waitStart >= 80, "datetime.waitFor should wait roughly the requested duration");

await datetime.waitUntil(new Date(0));

const deadline = Date.now() + 200;
const waitUntilStart = Date.now();
await datetime.waitUntil(new Date(deadline));
assert(Date.now() - waitUntilStart >= 150, "datetime.waitUntil should wait until a near-future deadline");

await assertRejectsContains(
  async () => {
    const task = datetime.waitFor("10s");
    task.cancel();
    await task;
  },
  "Cancelled",
  "datetime.waitFor task should be cancellable",
);

await assertRejectsContains(
  async () => {
    const task = datetime.waitUntil(new Date(Date.now() + 10000));
    task.cancel();
    await task;
  },
  "Cancelled",
  "datetime.waitUntil task should be cancellable",
);

await assertRejectsContains(
  async () => {
    const task = datetime.waitForSchedule({ dayOfWeek: DayOfWeek.Monday });
    task.cancel();
    await task;
  },
  "Cancelled",
  "datetime.waitForSchedule task should be cancellable",
);

await assertRejectsContains(
  () => datetime.waitForSchedule({ hour: 24 }),
  "hour",
  "datetime.waitForSchedule should reject invalid hour values",
);

await assertRejectsContains(
  () => datetime.waitForSchedule({ minute: 60 }),
  "minute",
  "datetime.waitForSchedule should reject invalid minute values",
);

await assertRejectsContains(
  () => datetime.waitForSchedule({ dayOfMonth: 0 }),
  "day",
  "datetime.waitForSchedule should reject invalid dayOfMonth values",
);

assertEq(DayOfWeek.Monday, "Monday", "DayOfWeek enum should be exposed to scripts");
