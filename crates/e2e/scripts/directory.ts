const tmp = standardPaths.temp;
assert(tmp !== null, "temp path should be available");

const testDir = tmp + "/actiona4_e2e_dir_test";
const subDir = testDir + "/sub";

// Create nested directories
await Directory.create(subDir);
assert(await Filesystem.isDirectory(subDir), "nested dir should exist");

// Create a file inside
await File.writeText(testDir + "/hello.txt", "hello");

// listEntries
const entries = await Directory.listEntries(testDir);
assert(entries.length === 2, `expected 2 entries, got ${entries.length}`);

const names = entries.map((e) => e.fileName).sort();
assert(names.includes("hello.txt"), "entries should include hello.txt");
assert(names.includes("sub"), "entries should include sub");

// Entry types
const fileEntry = entries.find((e) => e.fileName === "hello.txt");
const dirEntry = entries.find((e) => e.fileName === "sub");
assert(fileEntry !== undefined, "file entry found");
assert(dirEntry !== undefined, "dir entry found");
assert(fileEntry!.isFile, "hello.txt should be a file");
assert(!fileEntry!.isDirectory, "hello.txt should not be a directory");
assert(dirEntry!.isDirectory, "sub should be a directory");
assert(!dirEntry!.isFile, "sub should not be a file");

// Remove recursively
await Directory.remove(testDir);
assert(!(await Filesystem.exists(testDir)), "dir should be gone after remove");
