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

// clear() should not throw
clipboard.text.set("to be cleared");
clipboard.clear();
// After clear(), get() would throw because clipboard is empty — that is correct
// behaviour; we just verify clear() itself doesn't throw.
