// Create a 10x10 image
const img = new Image(10, 10);
assertEq(img.width, 10, "img.width");
assertEq(img.height, 10, "img.height");
assertEq(img.size.width, 10, "img.size.width");
assertEq(img.size.height, 10, "img.size.height");

// rect property
assertEq(img.rect.x, 0, "img.rect.x");
assertEq(img.rect.y, 0, "img.rect.y");
assertEq(img.rect.width, 10, "img.rect.width");
assertEq(img.rect.height, 10, "img.rect.height");

// fill and getPixel
img.fill(Color.Red);
const px = img.getPixel(0, 0);
assertEq(px.r, 255, "filled pixel r");
assertEq(px.g, 0, "filled pixel g");
assertEq(px.b, 0, "filled pixel b");

// setPixel / getPixel roundtrip
img.setPixel(new Point(5, 5), Color.Blue);
const px2 = img.getPixel(5, 5);
assertEq(px2.r, 0, "set pixel r");
assertEq(px2.g, 0, "set pixel g");
assertEq(px2.b, 255, "set pixel b");

// clone
const clone = img.clone();
assert(img.equals(clone), "clone equals original");
assert(!img.equals(new Image(10, 10)), "unfilled image differs from filled");

// filled (non-mutating)
const filled = img.filled(Color.Green);
const fpx = filled.getPixel(0, 0);
assertEq(fpx.g, 128, "filled green g=128");

// invertColors then getPixel
const inv = img.filled(Color.White).invertColors();
const ipx = inv.getPixel(0, 0);
assertEq(ipx.r, 0, "inverted white is black r=0");
assertEq(ipx.g, 0, "inverted white is black g=0");
assertEq(ipx.b, 0, "inverted white is black b=0");

// resize
const resized = new Image(10, 10).filled(Color.Red).resized(20, 20);
assertEq(resized.width, 20, "resized width");
assertEq(resized.height, 20, "resized height");

// toString includes dimensions
const str = img.toString();
assert(str.includes("10"), "toString includes dimensions");
