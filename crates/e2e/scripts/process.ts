function shellArgs(expr: string): string[] {
  return system.isWindows ? ["/c", expr] : ["-c", expr];
}

function shellProgram(): string {
  return system.isWindows ? "cmd" : "sh";
}

function longSleepHandle() {
  return system.isWindows
    ? process.start("ping", { args: ["-n", "100", "127.0.0.1"] })
    : process.start("sleep", { args: ["100"] });
}

function shortDetachedPid(): number {
  return system.isWindows
    ? process.startDetached("ping", { args: ["-n", "1", "127.0.0.1"] })
    : process.startDetached("sleep", { args: ["0.1"] });
}

// start() + stdout iteration
{
  const handle = process.start(shellProgram(), { args: shellArgs("echo hello world") });
  let output = "";
  for await (const line of handle.stdout) {
    output += line;
  }
  const closed = await handle.closed;
  assertEq(closed.exitCode, 0, "echo handle should exit 0");
  assert(output.trim() === "hello world", `stdout should be 'hello world', got ${JSON.stringify(output.trim())}`);
}

// stdin -> stdout roundtrip
{
  const handle = system.isWindows
    ? process.start("findstr", { args: [".*"] })
    : process.start("cat");
  await handle.write("test input\n");
  await handle.closeStdin();
  let output = "";
  for await (const line of handle.stdout) {
    output += line;
  }
  const closed = await handle.closed;
  assertEq(closed.exitCode, 0, "stdin roundtrip command should exit 0");
  assertEq(output.trim(), "test input", "stdin should be echoed to stdout");
}

// exit code
{
  const handle = process.start(shellProgram(), { args: shellArgs("exit 42") });
  const closed = await handle.closed;
  assertEq(closed.exitCode, 42, "closed.exitCode should forward the process exit code");
}

// stderr
{
  const expr = system.isWindows ? "echo error 1>&2" : "echo error >&2";
  const handle = process.start(shellProgram(), { args: shellArgs(expr) });
  let output = "";
  for await (const line of handle.stderr) {
    output += line;
  }
  const closed = await handle.closed;
  assertEq(closed.exitCode, 0, "stderr command should exit 0");
  assertEq(output.trim(), "error", "stderr should be captured");
}

// kill()
{
  const handle = longSleepHandle();
  handle.kill();
  await handle.closed;
}

// startDetached()
{
  const pid = shortDetachedPid();
  assert(pid > 0, "startDetached should return a positive pid");
}

// pid
{
  const handle = process.start(shellProgram(), { args: shellArgs("echo test") });
  assert(handle.pid > 0, "process handle pid should be positive");
  await handle.closed;
}

// startAndWait stdout
{
  const result = await process.startAndWait(shellProgram(), { args: shellArgs("echo hello world") });
  assertEq(result.exitCode, 0, "startAndWait echo should exit 0");
  if (result.stdout === undefined) {
    throw new Error("stdout should be captured for startAndWait");
  }
  assertEq(result.stdout.trim(), "hello world", "startAndWait stdout");
}

// startAndWait exit code
{
  const failing = await process.startAndWait(shellProgram(), { args: shellArgs("exit 7") });
  assertEq(failing.exitCode, 7, "startAndWait exit code should be 7");
}

// startAndWait stderr
{
  const expr = system.isWindows ? "echo err 1>&2" : "echo err >&2";
  const result = await process.startAndWait(shellProgram(), { args: shellArgs(expr) });
  assertEq(result.exitCode, 0, "startAndWait stderr command should exit 0");
  if (result.stderr === undefined) {
    throw new Error("stderr should be captured for startAndWait");
  }
  assertEq(result.stderr.trim(), "err", "startAndWait stderr");
}

// terminate()
{
  const handle = longSleepHandle();
  handle.terminate();
  await handle.closed;
}

// shell()
{
  const exitCode = await process.shell("exit 42");
  assertEq(exitCode, 42, "process.shell should forward exit codes");
}

{
  const exitCode = await process.shell("echo hello from shell");
  assertEq(exitCode, 0, "process.shell should succeed for a simple command");
}

// Windows GUI terminate regression
if (system.isWindows) {
  const handle = process.start("charmap");
  await sleep("2s");
  handle.terminate();
  await handle.closed;
}
