// Constructor and properties
const p = new Point(3, 4);
assertEq(p.x, 3, "p.x");
assertEq(p.y, 4, "p.y");

const p1 = new Point({ x: 1, y: 2 });
const p2 = new Point(2, 3);
const p3 = new Point(p2);
assert(!(p1 == p2), "Point == should compare identity, not value");
assert(p1 != p2, "Point != should differ for distinct instances");
assert(p2.equals(p3), "Point.equals should compare values");

// length() = distance from origin
assertEq(p.length(), 5, "length of (3,4) should be 5");

// distanceTo()
const origin = new Point(0, 0);
assertEq(origin.distanceTo(p), 5, "distanceTo (3,4) from origin");

// Static distance
assertEq(Point.distance(origin, p), 5, "Point.distance");

// isOrigin
assert(origin.isOrigin(), "origin.isOrigin()");
assert(!p.isOrigin(), "p.isOrigin() should be false");

// Zero constant
assertEq(Point.Zero.x, 0, "Point.Zero.x");
assertEq(Point.Zero.y, 0, "Point.Zero.y");

// toJson
const json = JSON.parse(p.toJson());
assertEq(json.x, 3, "json.x");
assertEq(json.y, 4, "json.y");

// Construct from PointLike plain object
const pointLike = new Point({ x: 10, y: 20 });
assertEq(pointLike.x, 10, "constructed PointLike x");
assertEq(pointLike.y, 20, "constructed PointLike y");

// add / subtract / scale
const added = p1.add(new Point(1, 3));
assertEq(added.x, 2, "added.x");
assertEq(added.y, 5, "added.y");
const subtracted = p1.subtract(new Point(1, 3));
assertEq(subtracted.x, 0, "subtracted.x");
assertEq(subtracted.y, -1, "subtracted.y");
const scaled = p1.scaled(2);
assertEq(scaled.x, 2, "scaled.x");
assertEq(scaled.y, 4, "scaled.y");

// clone
const clone = p1.clone();
assert(clone.equals(p1), "clone should preserve value equality");
assert(!(clone == p1), "clone should not preserve JS identity");

// randomInCircle returns a point within radius
const center = new Point(100, 100);
const rnd = Point.randomInCircle(center, 50);
assert(center.distanceTo(rnd) <= 50, "randomInCircle within radius");
