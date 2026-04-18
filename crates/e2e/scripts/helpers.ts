function assert(condition: boolean, message: string): void {
    if (!condition) throw new Error(`Assertion failed: ${message}`);
}

function assertEq<T>(actual: T, expected: T, message: string): void {
    if (actual !== expected)
        throw new Error(
            `Assertion failed: ${message}\n  expected: ${JSON.stringify(expected)}\n  actual:   ${JSON.stringify(actual)}`
        );
}

function assertMatches(value: string, pattern: RegExp, message: string): void {
    if (!pattern.test(value))
        throw new Error(
            `Assertion failed: ${message}\n  value: ${JSON.stringify(value)}\n  pattern: ${pattern}`
        );
}

function assertInRange(value: number, min: number, max: number, message: string): void {
    if (value < min || value > max)
        throw new Error(
            `Assertion failed: ${message}\n  expected [${min}, ${max}], got ${value}`
        );
}

async function assertRejectsContains(
    action: () => Promise<unknown> | unknown,
    expectedSubstring: string,
    message: string
): Promise<void> {
    try {
        await action();
    } catch (error) {
        const text = String(error);
        if (text.includes(expectedSubstring)) return;
        throw new Error(
            `Assertion failed: ${message}\n  expected error containing: ${JSON.stringify(expectedSubstring)}\n  actual error: ${JSON.stringify(text)}`
        );
    }

    throw new Error(`Assertion failed: ${message}\n  expected action to throw`);
}
