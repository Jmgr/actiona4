// @guard: e2e::require_not_wayland!();

// all() returns an array (may be empty in headless CI, but shouldn't throw)
const all = windows.all();
assert(Array.isArray(all), "windows.all() returns an array");

// active() should return some window handle without throwing
const active = windows.active();
assert(active !== undefined, "windows.active() returns a handle");

// WindowHandle has a title() method
const title = active.title();
assert(typeof title === "string", "active window title is a string");

// verify that find() with an unlikely title returns an empty array
const byTitle = windows.find({ title: "__actiona4_e2e_no_such_window__" });
assert(Array.isArray(byTitle), "find() returns an array");
assertEq(byTitle.length, 0, "find with bogus title returns empty");
