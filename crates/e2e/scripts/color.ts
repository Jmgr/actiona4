// Basic Color construction
const red = new Color(255, 0, 0);
assertEq(red.r, 255, "red.r should be 255");
assertEq(red.g, 0, "red.g should be 0");
assertEq(red.b, 0, "red.b should be 0");
assertEq(red.a, 255, "red.a defaults to 255");

// With alpha
const semi = new Color(0, 128, 255, 64);
assertEq(semi.r, 0, "semi.r");
assertEq(semi.g, 128, "semi.g");
assertEq(semi.b, 255, "semi.b");
assertEq(semi.a, 64, "semi.a");

// Named constants
assertEq(Color.Red.r, 255, "Color.Red.r");
assertEq(Color.Red.g, 0, "Color.Red.g");
assertEq(Color.Red.b, 0, "Color.Red.b");

assertEq(Color.Green.r, 0, "Color.Green.r");
assertEq(Color.Green.g, 128, "Color.Green.g");
assertEq(Color.Green.b, 0, "Color.Green.b");

assertEq(Color.Blue.r, 0, "Color.Blue.r");
assertEq(Color.Blue.g, 0, "Color.Blue.g");
assertEq(Color.Blue.b, 255, "Color.Blue.b");

assertEq(Color.White.r, 255, "Color.White.r");
assertEq(Color.White.g, 255, "Color.White.g");
assertEq(Color.White.b, 255, "Color.White.b");

assertEq(Color.Black.r, 0, "Color.Black.r");
assertEq(Color.Black.g, 0, "Color.Black.g");
assertEq(Color.Black.b, 0, "Color.Black.b");

assertEq(Color.Transparent.a, 0, "Color.Transparent.a");
assert(Color.Aqua.equals(Color.Cyan), "Color.Aqua should alias Color.Cyan");
assert(Color.Fuchsia.equals(Color.Magenta), "Color.Fuchsia should alias Color.Magenta");

// Construct from ColorLike plain object
const fromObj = new Color({ r: 10, g: 20, b: 30 });
assertEq(fromObj.r, 10, "fromObj.r");
assertEq(fromObj.g, 20, "fromObj.g");
assertEq(fromObj.b, 30, "fromObj.b");

const fromColor = new Color(Color.Red);
assertEq(fromColor.r, 255, "new Color(Color.Red).r");
assertEq(fromColor.a, 255, "new Color(Color.Red).a");

const clone = Color.Red.clone();
clone.g = 10;
clone.b = 11;
clone.a = 12;
assertEq(clone.r, 255, "clone.r");
assertEq(clone.g, 10, "clone.g");
assertEq(clone.b, 11, "clone.b");
assertEq(clone.a, 12, "clone.a");
assert(!clone.equals(Color.Red), "mutated clone should no longer equal Color.Red");
assertEq(
    clone.toString(),
    "Color(r: 255, g: 10, b: 11, a: 12)",
    "Color.toString() should reflect all channels",
);
assert(clone.clone().equals(clone), "clone().equals(clone) should be true");
assert(Color.Red.clone().equals(Color.Red), "Color.clone should preserve value equality");
assert(!(Color.Red.clone() == Color.Red), "Color.clone should not preserve JS identity");
