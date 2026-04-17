// Constructor and properties
const s = new Size(100, 50);
assertEq(s.width, 100, "s.width");
assertEq(s.height, 50, "s.height");

// equals
const s2 = new Size(100, 50);
assert(s.equals(s2), "s.equals(s2)");
assert(!s.equals(new Size(99, 50)), "different sizes not equal");

// add
const sum = s.add(new Size(20, 10));
assertEq(sum.width, 120, "sum.width");
assertEq(sum.height, 60, "sum.height");

// subtract
const diff = s.subtract(new Size(30, 20));
assertEq(diff.width, 70, "diff.width");
assertEq(diff.height, 30, "diff.height");

// scale
const scaled = s.scale(2);
assertEq(scaled.width, 200, "scaled.width");
assertEq(scaled.height, 100, "scaled.height");

// clone
const cloned = s.clone();
assert(s.equals(cloned), "cloned equals original");

// toJson
const json = JSON.parse(s.toJson());
assertEq(json.width, 100, "json.width");
assertEq(json.height, 50, "json.height");

// Construct from SizeLike
const s3 = new Size({ width: 5, height: 3 });
assertEq(s3.width, 5, "s3.width");
assertEq(s3.height, 3, "s3.height");
