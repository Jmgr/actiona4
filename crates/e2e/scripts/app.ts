assert(typeof app.version === "string", "app.version should be a string");
assert(app.version.length > 0, "app.version should not be empty");
assertMatches(app.version, /^\d+\.\d+\.\d+/, "app.version should match semver");

const env = app.env;
assert(typeof env === "object" && env !== null, "app.env should be an object");

const cwd = app.cwd;
assert(typeof cwd === "string", "app.cwd should be a string");
assert(cwd.length > 0, "app.cwd should not be empty");

const exePath = app.executablePath;
assert(typeof exePath === "string", "app.executablePath should be a string");
