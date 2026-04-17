// Constructor and properties
const p = new Point(3, 4);
assertEq(p.x, 3, "p.x");
assertEq(p.y, 4, "p.y");

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
const p2 = new Point({ x: 10, y: 20 });
assertEq(p2.x, 10, "p2.x");
assertEq(p2.y, 20, "p2.y");

// randomInCircle returns a point within radius
const center = new Point(100, 100);
const rnd = Point.randomInCircle(center, 50);
assert(center.distanceTo(rnd) <= 50, "randomInCircle within radius");
