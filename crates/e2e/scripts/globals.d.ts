declare function assert(condition: boolean, message: string): void;
declare function assertEq<T>(actual: T, expected: T, message: string): void;
declare function assertMatches(value: string, pattern: RegExp, message: string): void;
declare function assertInRange(value: number, min: number, max: number, message: string): void;
declare function assertRejectsContains(
    action: () => Promise<unknown> | unknown,
    expectedSubstring: string,
    message: string,
): Promise<void>;
