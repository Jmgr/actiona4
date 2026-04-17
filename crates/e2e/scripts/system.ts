// CPU
const cpu = system.cpu;
assert(cpu.logicalCoreCount >= 1, "logicalCoreCount >= 1");
assert(typeof cpu.architecture === "string" && cpu.architecture.length > 0, "architecture is a non-empty string");

const usage = await cpu.usage();
assertInRange(usage, 0, 100, "cpu usage in [0, 100]");

const freqs = await cpu.frequencies();
assert(Array.isArray(freqs), "frequencies is an array");
assert(freqs.length >= 1, "at least one frequency");
for (const f of freqs) {
  assert(f >= 0, "frequency >= 0");
}

// OS
const os = system.os;
assert(typeof os.distributionId === "string", "distributionId is a string");
assert(os.uptime >= 0, "uptime >= 0");
assert(os.bootTime instanceof Date, "bootTime is a Date");
assert(os.kernelLongVersion.length > 0, "kernelLongVersion is non-empty");

// Memory
const memUsage = await system.memory.usage();
assert(memUsage.total > 0, "total memory > 0");
assert(memUsage.used >= 0, "used >= 0");
assert(memUsage.free >= 0, "free >= 0");
assert(memUsage.available >= 0, "available >= 0");
assert(memUsage.used + memUsage.free <= memUsage.total + 1, "used + free <= total");

// Formatting helpers
const formatted = formatBytes(1048576);
assert(typeof formatted === "string" && formatted.length > 0, "formatBytes returns non-empty string");

const percent = formatPercent(75.5);
assert(percent.includes("%"), "formatPercent includes %");

const freq = formatFrequency(1000000);
assert(freq.includes("MHz") || freq.includes("kHz") || freq.includes("Hz"), "formatFrequency includes unit");
