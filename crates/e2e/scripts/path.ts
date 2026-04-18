// join assembles path segments
const joined = Path.join("a", "b", "c");
assert(joined.includes("b"), "Path.join should include middle segment");
assertEq(
    Path.join("foo", "bar"),
    system.isWindows ? "foo\\bar" : "foo/bar",
    "Path.join should use the current platform separator",
);

// filename / basename both return the last segment including extension
assertEq(Path.filename("/some/dir/file.txt"), "file.txt", "Path.filename");
assertEq(Path.basename("/some/dir/file.txt"), "file.txt", "Path.basename is alias for filename");
assertEq(Path.filename("/foo/bar"), "bar", "Path.filename should return the last path segment");

// parent / dirname return parent directory
const parent = Path.parent("/some/dir/file.txt");
assert(
    parent.endsWith("dir") || parent.endsWith("dir/") || parent.endsWith("dir\\"),
    "Path.parent should return parent dir"
);
assertEq(Path.parent("/foo/bar/test.txt"), "/foo/bar", "Path.parent of a file path");
assertEq(Path.parent("/foo/bar"), "/foo", "Path.parent of a directory path");

// extension and extname both return the extension without a leading dot
assertEq(Path.extension("/some/dir/file.txt"), "txt", "Path.extension");
assertEq(Path.extname("/some/dir/file.txt"), "txt", "Path.extname is alias for extension");
assertEq(Path.extension("foo/bar"), "", "Path.extension should be empty when no extension exists");

// isAbsolute / isRelative
const absolutePath = system.isWindows ? "C:/absolute/path" : "/absolute/path";
assert(Path.isAbsolute(absolutePath), `Path.isAbsolute(${JSON.stringify(absolutePath)})`);
assert(!Path.isRelative(absolutePath), `not Path.isRelative(${JSON.stringify(absolutePath)})`);
assert(Path.isRelative("relative/path"), "Path.isRelative('relative/path')");
assert(!Path.isAbsolute("relative/path"), "not Path.isAbsolute('relative/path')");

// setExtension replaces the extension
const newPath = Path.setExtension("/some/dir/file.txt", "rs");
assertMatches(newPath, /\.rs$/, "Path.setExtension should change extension to .rs");
assertEq(
    Path.setExtension("/foo/bar/test.txt", "foo"),
    "/foo/bar/test.foo",
    "Path.setExtension should replace an existing extension",
);
assertEq(
    Path.setExtension("/foo/bar/test", "foo"),
    "/foo/bar/test.foo",
    "Path.setExtension should append an extension when there is none",
);
assertEq(
    Path.setExtension("/foo/bar/test", "not/valid"),
    "",
    "Path.setExtension should reject invalid extensions",
);
