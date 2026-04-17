// @guard: e2e::require_not_windows!();

// startAndWait: echo hello
const result = await process.startAndWait("echo", { args: ["hello"] });
assertEq(result.exitCode, 0, "echo should exit 0");
if (result.stdout === undefined) {
  throw new Error("stdout should be captured for startAndWait");
}
const stdout = result.stdout.trim();
assert(stdout === "hello", `stdout should be 'hello', got '${stdout}'`);

// startAndWait: failing command via sh
const failing = await process.startAndWait("sh", { args: ["-c", "exit 42"] });
assertEq(failing.exitCode, 42, "exit code should be 42");

// start + iterate stdout
const handle = process.start("echo", { args: ["line1"] });
const lines: string[] = [];
for await (const line of handle.stdout) {
  lines.push(line);
}
await handle.closed;
assert(lines.some((l) => l.includes("line1")), "should have captured 'line1' from stdout");
