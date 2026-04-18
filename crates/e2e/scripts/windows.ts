if (system.platform === "wayland") { println("skipping: not supported on Wayland"); exit(); }

function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// all() returns an array (may be empty in headless CI, but shouldn't throw)
const all = windows.all();
assert(Array.isArray(all), "windows.all() returns an array");

const anyTitle = windows.find({ title: /.*/ });
assert(Array.isArray(anyTitle), "find({ title: /.*/ }) returns an array");
for (const win of anyTitle) {
  assertMatches(win.title(), /.*/, "title filter should match every returned title");
}

const anyClass = windows.find({ className: /.*/ });
assert(Array.isArray(anyClass), "find({ className: /.*/ }) returns an array");
for (const win of anyClass) {
  assertMatches(win.className(), /.*/, "className filter should match every returned class name");
}

const visibleWindows = windows.find({ visible: true });
assert(Array.isArray(visibleWindows), "find({ visible: true }) returns an array");
for (const win of visibleWindows) {
  assert(win.isVisible(), "visible filter should only return visible windows");
}

// active() and foreground() return undefined or the active WindowHandle
const active = windows.active();
const foreground = windows.foreground();
assertEq(
  active === undefined,
  foreground === undefined,
  "foreground() should agree with active() on whether a window is focused",
);

if (active !== undefined && foreground !== undefined) {
  assertEq(active.title(), foreground.title(), "foreground() should reference the same window title");
  assertEq(active.className(), foreground.className(), "foreground() should reference the same window class");
  assertEq(active.processId(), foreground.processId(), "foreground() should reference the same window process id");

  const title = active.title();
  assert(typeof title === "string", "active window title is a string");

  const className = active.className();
  assert(typeof className === "string", "active window className is a string");

  const processId = active.processId();
  assert(processId > 0, "active window processId should be positive");

  assert(active.isActive(), "active window should report itself as active");

  const position = active.position();
  const size = active.size();
  const rect = active.rect();
  assertEq(position.x, rect.x, "position().x should match rect().x");
  assertEq(position.y, rect.y, "position().y should match rect().y");
  assertEq(size.width, rect.width, "size().width should match rect().width");
  assertEq(size.height, rect.height, "size().height should match rect().height");

  const byPid = windows.find({ processId });
  assert(byPid.length > 0, "processId filter should find at least the active window");

  const byExactTitle = windows.find({ title });
  assert(byExactTitle.some(win => win.title() === title), "exact-string title matching should find the active window");

  const byExactClass = windows.find({ className });
  assert(byExactClass.some(win => win.className() === className), "exact-string className matching should find the active window");

  if (title.length > 0) {
    const titleWildcard = new Wildcard(`${title.slice(0, 1)}*`);
    const byTitleWildcard = windows.find({ title: titleWildcard });
    assert(
      byTitleWildcard.some(win => win.title() === title),
      "wildcard title matching should find the active window when the prefix matches",
    );

    const byTitleRegex = windows.find({ title: new RegExp(`^${escapeRegex(title)}$`) });
    assert(
      byTitleRegex.some(win => win.title() === title),
      "regex title matching should find the active window",
    );
  }

  if (className.length > 0) {
    const classWildcard = new Wildcard(`${className.slice(0, 1)}*`);
    const byClassWildcard = windows.find({ className: classWildcard });
    assert(
      byClassWildcard.some(win => win.className() === className),
      "wildcard className matching should find the active window when the prefix matches",
    );

    const byClassRegex = windows.find({ className: new RegExp(`^${escapeRegex(className)}$`) });
    assert(
      byClassRegex.some(win => win.className() === className),
      "regex className matching should find the active window",
    );
  }

  if (rect.width > 0 && rect.height > 0) {
    const center = new Point(
      Math.floor(rect.x + rect.width / 2),
      Math.floor(rect.y + rect.height / 2),
    );
    const byPoint = windows.findAt(center);
    assert(byPoint.length > 0, "findAt(center) should find at least one window covering the center point");

    for (const win of byPoint) {
      assert(win.rect().contains(center), "findAt() should only return windows whose rect contains the point");
    }
  }
}

// verify that find() with an unlikely title returns an empty array
const byTitle = windows.find({ title: "__actiona4_e2e_no_such_window__" });
assert(Array.isArray(byTitle), "find() returns an array");
assertEq(byTitle.length, 0, "find with bogus title returns empty");
