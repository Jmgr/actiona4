// Constructor and properties
const s = new Size(100, 50);
assertEq(s.width, 100, "s.width");
assertEq(s.height, 50, "s.height");

const s1 = new Size({ width: 1, height: 2 });
const s2 = new Size(2, 3);
const s3 = new Size(s2);
assert(!(s1 == s2), "Size == should compare identity, not value");
assert(s1 != s2, "Size != should differ for distinct instances");
assert(s2.equals(s3), "Size.equals should compare values");

// equals
const same = new Size(100, 50);
assert(s.equals(same), "s.equals(same)");
assert(!s.equals(new Size(99, 50)), "different sizes not equal");

// mutable attributes
s1.width = 42;
s1.height = 43;
assertEq(s1.width, 42, "mutable width");
assertEq(s1.height, 43, "mutable height");

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
assert(!(cloned == s), "clone should not preserve JS identity");

// toJson
const json = JSON.parse(s.toJson());
assertEq(json.width, 100, "json.width");
assertEq(json.height, 50, "json.height");

// Construct from SizeLike
const sizeLike = new Size({ width: 5, height: 3 });
assertEq(sizeLike.width, 5, "sizeLike.width");
assertEq(sizeLike.height, 3, "sizeLike.height");
