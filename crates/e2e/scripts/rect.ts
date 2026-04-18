// Constructor and properties
const r = new Rect(10, 20, 100, 80);
assertEq(r.x, 10, "r.x");
assertEq(r.y, 20, "r.y");
assertEq(r.width, 100, "r.width");
assertEq(r.height, 80, "r.height");

// topLeft and size
assertEq(r.topLeft.x, 10, "topLeft.x");
assertEq(r.topLeft.y, 20, "topLeft.y");
assertEq(r.size.width, 100, "size.width");
assertEq(r.size.height, 80, "size.height");

// equals
const r2 = new Rect(10, 20, 100, 80);
assert(r.equals(r2), "r.equals(r2)");

// contains
assert(r.contains(new Point(50, 50)), "contains inner point");
assert(!r.contains(new Point(5, 5)), "does not contain outer point");

// intersects
const r3 = new Rect(50, 50, 100, 100);
assert(r.intersects(r3), "overlapping rects intersect");
const r4 = new Rect(200, 200, 50, 50);
assert(!r.intersects(r4), "non-overlapping rects do not intersect");

// intersection
const inter = r.intersection(r3);
assert(inter !== undefined, "intersection should exist");
if (inter) {
  assertEq(inter.x, 50, "inter.x");
  assertEq(inter.y, 50, "inter.y");
  assertEq(inter.width, 60, "inter.width");
  assertEq(inter.height, 50, "inter.height");
}

// no intersection
const noInter = r.intersection(r4);
assert(noInter === undefined, "no intersection should be undefined");

// union
const u = new Rect(0, 0, 50, 50).union(new Rect(25, 25, 50, 50));
assertEq(u.x, 0, "union.x");
assertEq(u.y, 0, "union.y");
assertEq(u.width, 75, "union.width");
assertEq(u.height, 75, "union.height");

// clone
const cloned = r.clone();
assert(r.equals(cloned), "cloned equals original");

// Construct from RectLike
const r5 = new Rect({ x: 1, y: 2, width: 3, height: 4 });
assertEq(r5.x, 1, "r5.x");
assertEq(r5.width, 3, "r5.width");

const r6 = new Rect(r2);
assert(r6.equals(r2), "constructing from another Rect should preserve value equality");
