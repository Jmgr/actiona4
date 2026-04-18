// @guard: e2e::require_not_wayland!();

// all() returns at least one display
const all = displays.all();
assert(all.length >= 1, "at least one display");

// primary() returns a display
const primary = displays.primary();
assert(primary.rect.width > 0, "primary display width > 0");
assert(primary.rect.height > 0, "primary display height > 0");
assert(primary.isPrimary, "primary display should have isPrimary = true");

// smallest / largest return undefined or a display
const smallest = displays.smallest();
const largest = displays.largest();
if (smallest !== undefined) assert(smallest.rect.width > 0, "smallest display width > 0");
if (largest !== undefined) assert(largest.rect.width > 0, "largest display width > 0");

// fromPoint with a coordinate inside the primary display
const mid = {
  x: primary.rect.x + Math.floor(primary.rect.width / 2),
  y: primary.rect.y + Math.floor(primary.rect.height / 2),
};
const found = displays.fromPoint(mid.x, mid.y);
assert(found !== undefined, "fromPoint should find a display for primary midpoint");
