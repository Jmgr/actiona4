// Smoke-test: none of these should throw
console.log("log message");
console.warn("warn message");
console.error("error message");
console.info("info message");
console.log("number:", 42, "bool:", true, "null:", null);
console.log("object:", { a: 1, b: [2, 3] });
console.inspect({ nested: { deep: true } });

// time / timeEnd
console.time("label");
console.timeEnd("label");

// count
console.count("counter");
console.count("counter");

// clear should not throw
console.clear();
