// Basic text roundtrip
clipboard.text.set("hello actiona");
assertEq(clipboard.text.get(), "hello actiona", "clipboard text roundtrip");

// Unicode roundtrip
const unicode = "こんにちは🎉";
clipboard.text.set(unicode);
assertEq(clipboard.text.get(), unicode, "clipboard unicode roundtrip");

// HTML roundtrip
clipboard.html.set("<b>bold</b>");
const html = clipboard.html.get();
assert(html.includes("bold"), "clipboard HTML roundtrip should contain 'bold'");

clipboard.html.set("<b>test</b>", "test");
assertEq(clipboard.html.get(), "<b>test</b>", "clipboard HTML should roundtrip exactly");
assertEq(clipboard.text.get(), "test", "clipboard HTML plain-text fallback should be available");

// Image roundtrip
const image = new Image(128, 128);
image.setPixel(32, 32, new Color(16, 32, 64, 128));
clipboard.image.set(image);
const clipped = clipboard.image.get();
assertEq(clipped.width, 128, "clipboard image width");
assertEq(clipped.height, 128, "clipboard image height");
const pixel = clipped.getPixel(32, 32);
assertEq(pixel.r, 16, "clipboard image pixel.r");
assertEq(pixel.g, 32, "clipboard image pixel.g");
assertEq(pixel.b, 64, "clipboard image pixel.b");
assertEq(pixel.a, 128, "clipboard image pixel.a");

// clear() should not throw
clipboard.text.set("to be cleared");
clipboard.clear();
// After clear(), get() would throw because clipboard is empty — that is correct
// behaviour; we just verify clear() itself doesn't throw.
