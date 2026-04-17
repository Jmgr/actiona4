const tmp = standardPaths.temp;
assert(tmp !== null, "temp path should be available");
const testFile = tmp + "/actiona4_e2e_file_test.txt";

// Write and read back
await File.writeText(testFile, "hello world");
const text = await File.readText(testFile);
assertEq(text, "hello world", "read back should match written text");

// exists
assert(await File.exists(testFile), "file should exist after write");

// Unicode roundtrip
await File.writeText(testFile, "こんにちは 🎉");
const unicode = await File.readText(testFile);
assertEq(unicode, "こんにちは 🎉", "unicode roundtrip");

// Overwrite
await File.writeText(testFile, "second");
const second = await File.readText(testFile);
assertEq(second, "second", "overwrite should replace content");

// File handle: open, write, read, close
const handle = await File.open(testFile, { write: true, truncate: true });
await handle.writeText("via handle");
handle.close();

const handle2 = await File.open(testFile, { read: true });
const fromHandle = await handle2.readText();
handle2.close();
assertEq(fromHandle, "via handle", "file handle write/read roundtrip");

// size
const handle3 = await File.open(testFile, { read: true });
const sz = await handle3.size();
handle3.close();
assert(sz > 0, "file size should be positive");

// remove
await File.remove(testFile);
assert(!(await File.exists(testFile)), "file should not exist after remove");
