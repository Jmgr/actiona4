const raceStart = Date.now();
const raceResult = await concurrency.race([sleep("100ms").then(() => "fast"), sleep("1s").then(() => "slow")]);
const raceDuration = Date.now() - raceStart;
assertEq(raceResult, "fast", "concurrency.race should resolve with the first task");
assert(raceDuration < 1000, "concurrency.race should not wait for slower tasks");

const nestedRaceStart = Date.now();
const nestedRaceResult = await concurrency.race([
  concurrency.race([sleep("100ms").then(() => "nested-fast")]),
  sleep("1s").then(() => "nested-slow"),
]);
const nestedRaceDuration = Date.now() - nestedRaceStart;
assertEq(nestedRaceResult, "nested-fast", "nested concurrency.race should still resolve with the first task");
assert(nestedRaceDuration < 1000, "nested concurrency.race should not wait for slower tasks");

assert((await concurrency.race([])) === undefined, "concurrency.race([]) should resolve to undefined");

const ignoredNonPromises = await concurrency.race([1, "text", sleep("30ms").then(() => "done"), null]);
assertEq(ignoredNonPromises, "done", "concurrency.race should ignore non-promise values");

const rejectionResult = await (async (): Promise<string> => {
  const loser = sleep("1s");
  let loserError = "";
  loser.catch((error: unknown) => {
    loserError = String(error);
  });

  let raceError = "";
  try {
    await concurrency.race([
      sleep("20ms").then(() => {
        throw new Error("boom");
      }),
      loser,
    ]);
  } catch (error) {
    raceError = String(error);
  }

  await sleep("50ms");
  return `${raceError}|${loserError}`;
})();
assertEq(
  rejectionResult,
  "Error: boom|Error: Cancelled",
  "concurrency.race should propagate the first rejection and cancel losers",
);

const loserError = await (async (): Promise<string> => {
  const winner = sleep("30ms");
  const loser = sleep("1s");

  await concurrency.race([winner, loser]);
  await winner;

  try {
    await loser;
    return "";
  } catch (error) {
    return String(error);
  }
})();
assertEq(loserError, "Error: Cancelled", "concurrency.race should cancel losing tasks");
