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
assert(/^[\x00-\x7F]*$/.test(s), "random.string() should default to ASCII");

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
assertEq(c.a, 255, "random.color() should always be opaque");

// colorWithAlpha() has alpha too
const ca = random.colorWithAlpha();
assertInRange(ca.a, 0, 255, "random.colorWithAlpha().a in [0,255]");

let foundNonOpaqueAlpha = false;
for (let seed = 0; seed < 1024; seed++) {
  random.setSeed(seed);
  if (random.colorWithAlpha().a !== 255) {
    foundNonOpaqueAlpha = true;
    break;
  }
}
assert(foundNonOpaqueAlpha, "random.colorWithAlpha() should sometimes produce non-opaque alpha");

random.setSeed(1234);
const c1 = random.color();
random.setSeed(1234);
const c2 = random.color();
assert(c1.equals(c2), "random.color() should be deterministic when seeded");

random.setSeed(4567);
const s1 = random.string(24);
random.setSeed(4567);
const s2 = random.string(24);
assertEq(s1, s2, "random.string() should be deterministic when seeded");

const customChars = "ab12";
const custom = random.string(256, { characters: customChars });
assert([...custom].every(char => customChars.includes(char)), "random.string() should honor a custom character set");

await assertRejectsContains(
  () => Promise.resolve(random.string(4, { characters: "" })),
  "options.characters must not be empty",
  "random.string() should reject an empty character set",
);

const onlyDigits = random.string(128, {
  allowNumbers: true,
  allowLetters: false,
  allowSpecialCharacters: false,
});
assert(/^[0-9]+$/.test(onlyDigits), "boolean options should restrict random.string() output");

await assertRejectsContains(
  () => Promise.resolve(random.string(4, {
    allowNumbers: false,
    allowLetters: false,
    allowSpecialCharacters: false,
  })),
  "at least one of options.allowNumbers, options.allowLetters, options.allowSpecialCharacters must be true",
  "random.string() should reject all categories being disabled",
);

const overrideChars = random.string(128, {
  characters: "ab",
  allowNumbers: false,
  allowLetters: false,
  allowSpecialCharacters: false,
});
assert([...overrideChars].every(char => "ab".includes(char)), "characters should override boolean category options");

random.setSeed(314159);
const seededId1 = random.uuid();
random.setSeed(314159);
const seededId2 = random.uuid();
assertEq(seededId1, seededId2, "random.uuid() should be deterministic when seeded");

// Unicode character sets are accepted
const unicodeStr = random.string(4, { characters: "a\u0302\uD83D\uDC4D\uD83C\uDFFD\uD83C\uDDEC\uD83C\uDDE7" });
assert(typeof unicodeStr === "string" && unicodeStr.length > 0, "random.string() should accept unicode character sets");
