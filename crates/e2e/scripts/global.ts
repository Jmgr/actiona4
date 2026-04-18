// Global console alias: print
print("hello from print");

// Global console alias: println
println({ a: 1 });
println([1, 2]);

// Global console alias: inspect
inspect({ nested: [1, { value: "ok" }] });

// Global sleep
const sleepStart = Date.now();
await sleep("100ms");
assert(Date.now() - sleepStart >= 80, "global sleep should wait roughly the requested duration");

// exit() should not throw - must be last
exit();
