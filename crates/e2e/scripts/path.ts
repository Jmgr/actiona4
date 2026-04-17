// join assembles path segments
const joined = Path.join("a", "b", "c");
assert(joined.includes("b"), "Path.join should include middle segment");

// filename / basename both return the last segment including extension
assertEq(Path.filename("/some/dir/file.txt"), "file.txt", "Path.filename");
assertEq(Path.basename("/some/dir/file.txt"), "file.txt", "Path.basename is alias for filename");

// parent / dirname return parent directory
const parent = Path.parent("/some/dir/file.txt");
assert(
    parent.endsWith("dir") || parent.endsWith("dir/") || parent.endsWith("dir\\"),
    "Path.parent should return parent dir"
);

// extension and extname both return the extension without a leading dot
assertEq(Path.extension("/some/dir/file.txt"), "txt", "Path.extension");
assertEq(Path.extname("/some/dir/file.txt"), "txt", "Path.extname is alias for extension");

// isAbsolute / isRelative
assert(Path.isAbsolute("/absolute/path"), "Path.isAbsolute('/absolute/path')");
assert(!Path.isRelative("/absolute/path"), "not Path.isRelative('/absolute/path')");
assert(Path.isRelative("relative/path"), "Path.isRelative('relative/path')");
assert(!Path.isAbsolute("relative/path"), "not Path.isAbsolute('relative/path')");

// setExtension replaces the extension
const newPath = Path.setExtension("/some/dir/file.txt", "rs");
assertMatches(newPath, /\.rs$/, "Path.setExtension should change extension to .rs");
