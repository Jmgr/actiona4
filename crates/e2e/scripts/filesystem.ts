const tmp = standardPaths.temp;
assert(tmp !== null, "temp path should be available");

// The temp dir itself should exist and be a directory
assert(await Filesystem.exists(tmp!), "temp dir exists");
assert(await Filesystem.isDirectory(tmp!), "temp dir is a directory");
assert(!(await Filesystem.isFile(tmp!)), "temp dir is not a file");

// Non-existent path
const missing = tmp + "/actiona4_e2e_no_such_file_xyz";
assert(!(await Filesystem.exists(missing)), "missing path does not exist");
assert(!(await Filesystem.isFile(missing)), "missing path is not a file");
assert(!(await Filesystem.isDirectory(missing)), "missing path is not a directory");

// Create a file, verify it shows up correctly
const testFile = tmp + "/actiona4_e2e_fs_test.txt";
await File.writeText(testFile, "fs test");
assert(await Filesystem.exists(testFile), "file exists after write");
assert(await Filesystem.isFile(testFile), "written path is a file");
assert(!(await Filesystem.isDirectory(testFile)), "written path is not a directory");

// Cleanup
await File.remove(testFile);
