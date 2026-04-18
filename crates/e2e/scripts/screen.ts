if (system.platform === "wayland") { println("skipping: not supported on Wayland"); exit(); }

const primary = displays.primary()!;

const img = await screen.captureDesktop();
assert(img.width > 0, "captureDesktop width > 0");
assert(img.height > 0, "captureDesktop height > 0");

// captureRect with a small region
const small = await screen.captureRect({ x: 0, y: 0, width: 10, height: 10 });
assertEq(small.width, 10, "captureRect width should be 10");
assertEq(small.height, 10, "captureRect height should be 10");

// capturePixel returns a Color
const pixel = await screen.capturePixel({ x: 0, y: 0 });
assertInRange(pixel.r, 0, 255, "capturePixel.r");
assertInRange(pixel.g, 0, 255, "capturePixel.g");
assertInRange(pixel.b, 0, 255, "capturePixel.b");

assertEq(SearchIn.desktop().toString(), "SearchIn(desktop)", "SearchIn.desktop().toString()");
assert(
  SearchIn.display(primary).toString().startsWith("SearchIn(display_id:"),
  "SearchIn.display(...).toString() should include the display id prefix",
);
assertEq(
  SearchIn.rect(0, 0, 1920, 1080).toString(),
  "SearchIn(rect: (x: 0, y: 0, width: 1920, height: 1080))",
  "SearchIn.rect(...).toString()",
);
