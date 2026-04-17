// number() with no args returns [0, 1)
for (let i = 0; i < 20; i++) {
  const n = random.number();
  assertInRange(n, 0, 1, "random.number() should be in [0, 1]");
}

// number(min, max) returns value in [min, max]
for (let i = 0; i < 20; i++) {
  const n = random.number(5, 10);
  assertInRange(n, 5, 10, "random.number(5, 10) should be in [5, 10]");
}

// integer(min, max) returns inclusive range
for (let i = 0; i < 20; i++) {
  const n = random.integer(1, 6);
  assertInRange(n, 1, 6, "random.integer(1, 6) should be in [1, 6]");
  assertEq(Math.floor(n), n, "random.integer should be a whole number");
}

// string returns correct length
const s = random.string(12);
assert(typeof s === "string", "random.string should return a string");
assertEq(s.length, 12, "random.string(12) should have length 12");

// uuid returns correct format
const id = random.uuid();
assertMatches(
  id,
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
  "random.uuid should match UUID format",
);

// Two uuids should be different (with overwhelming probability)
const id2 = random.uuid();
assert(id !== id2, "two consecutive uuids should differ");

// color() returns a Color object with r/g/b/a in [0, 255]
const c = random.color();
assertInRange(c.r, 0, 255, "random.color().r in [0,255]");
assertInRange(c.g, 0, 255, "random.color().g in [0,255]");
assertInRange(c.b, 0, 255, "random.color().b in [0,255]");

// colorWithAlpha() has alpha too
const ca = random.colorWithAlpha();
assertInRange(ca.a, 0, 255, "random.colorWithAlpha().a in [0,255]");
