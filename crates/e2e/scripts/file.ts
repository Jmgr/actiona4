const tmp = standardPaths.temp;
assert(tmp !== null, "temp path should be available");
const testFile = tmp + `/actiona4_e2e_file_test_${random.uuid()}.txt`;
const copyFile = tmp + `/actiona4_e2e_file_copy_${random.uuid()}.txt`;
const renamedFile = tmp + `/actiona4_e2e_file_renamed_${random.uuid()}.txt`;

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
assert(await handle.isOpen(), "handle should start open");
await handle.writeText("via handle");
await handle.rewind();
assertEq(await handle.readText(), "via handle", "same handle can rewind and read");
handle.close();
assert(!(await handle.isOpen()), "handle should report closed after close()");
await assertRejectsContains(
    () => handle.setSize(42),
    "File is not open",
    "mutating a closed handle should fail",
);

const handle2 = await File.open(testFile, { read: true });
const fromHandle = await handle2.readText();
handle2.close();
assertEq(fromHandle, "via handle", "file handle write/read roundtrip");

// size
const handle3 = await File.open(testFile, { read: true });
const sz = await handle3.size();
handle3.close();
assert(sz > 0, "file size should be positive");

// setSize / position
const handle4 = await File.open(testFile, { write: true, read: true, truncate: true });
await handle4.writeText("test");
assertEq(await handle4.size(), 4, "size() should match written text length");
await handle4.setSize(42);
assertEq(await handle4.size(), 42, "setSize() should update the file size");
assertEq(await handle4.position(), 4, "position should advance after writing");
await handle4.setPosition(2);
assertEq(await handle4.position(), 2, "setPosition() should move the cursor");
await handle4.setRelativePosition(1);
assertEq(await handle4.position(), 3, "setRelativePosition(+1) should move forward");
await handle4.setRelativePosition(-1);
assertEq(await handle4.position(), 2, "setRelativePosition(-1) should move backward");
await handle4.rewind();
assertEq(await handle4.position(), 0, "rewind() should reset the cursor");
handle4.close();

// path / exists
const handle5 = await File.open(testFile, { read: true });
assertEq(handle5.path, testFile, "file.path should expose the opened path");
handle5.close();
assert(!(await File.exists(tmp + `/actiona4_e2e_missing_${random.uuid()}.txt`)), "missing file should not exist");

// readonly
const handle6 = await File.open(testFile, { read: true, write: true });
assert(!(await handle6.readonly()), "file should not start readonly");
await handle6.setReadonly(true);
assert(await handle6.readonly(), "setReadonly(true) should mark the file readonly");
await handle6.setReadonly(false);
assert(!(await handle6.readonly()), "setReadonly(false) should clear readonly");

// timestamps
const timestamp = new Date(Date.UTC(1996, 2, 10, 6, 46, 16, 468));
await handle6.setModifiedTime(timestamp);
const modified = await handle6.modifiedTime();
assertEq(modified.toISOString(), timestamp.toISOString(), "modifiedTime should roundtrip");
await handle6.setAccessedTime(timestamp);
const accessed = await handle6.accessedTime();
assertEq(accessed.toISOString(), timestamp.toISOString(), "accessedTime should roundtrip");
if (system.isWindows) {
  await handle6.setCreationTime(timestamp);
  const created = await handle6.creationTime();
  assertEq(created.toISOString(), timestamp.toISOString(), "creationTime should roundtrip on Windows");
}

// mode
if (system.isWindows) {
  await assertRejectsContains(
    () => handle6.mode(),
    "not supported on Windows",
    "mode() should be unsupported on Windows",
  );
  await assertRejectsContains(
    () => handle6.setMode(0o445),
    "not supported on Windows",
    "setMode() should be unsupported on Windows",
  );
} else {
  await handle6.setMode(0o445);
  assertEq((await handle6.mode()) & 0o777, 0o445, "setMode() should update the permission bits");
  await handle6.setMode(0o644);
}
handle6.close();

// copy / rename
await File.copy(testFile, copyFile);
assertEq(await File.readText(copyFile), await File.readText(testFile), "File.copy should preserve contents");
await File.rename(copyFile, renamedFile);
assert(!(await File.exists(copyFile)), "File.rename should remove the old path");
assertEq(await File.readText(renamedFile), await File.readText(testFile), "File.rename should preserve contents");
await File.remove(renamedFile);

// remove
await File.remove(testFile);
assert(!(await File.exists(testFile)), "file should not exist after remove");

// writeBytes / readBytes (static)
const bytesPath = tmp + `/actiona4_e2e_file_bytes_${random.uuid()}.bin`;
const testBytes = new Uint8Array([116, 101, 115, 116]); // "test"
await File.writeBytes(bytesPath, testBytes);
const readBytes = await File.readBytes(bytesPath);
assertEq(JSON.stringify(Array.from(readBytes)), JSON.stringify(Array.from(testBytes)), "File.writeBytes/readBytes static roundtrip");
await File.remove(bytesPath);

// writeBytes / readBytes (instance methods)
const bytesPath2 = tmp + `/actiona4_e2e_file_bytes2_${random.uuid()}.bin`;
const handle7 = await File.open(bytesPath2, { read: true, write: true, createNew: true });
await handle7.writeBytes(new Uint8Array([116, 101, 115, 116]));
await handle7.rewind();
const allBytes = await handle7.readBytes();
assertEq(JSON.stringify(Array.from(allBytes)), JSON.stringify([116, 101, 115, 116]), "instance writeBytes/readBytes roundtrip");
await handle7.rewind();
const twoBytes = await handle7.readBytes(2);
assertEq(JSON.stringify(Array.from(twoBytes)), JSON.stringify([116, 101]), "readBytes(2) returns only 2 bytes");
handle7.close();
await File.remove(bytesPath2);
