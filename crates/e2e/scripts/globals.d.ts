declare function assert(condition: unknown, message: string): asserts condition;
declare function assertEq<T>(actual: T, expected: T, message: string): void;
declare function assertMatches(value: string, pattern: RegExp, message: string): void;
declare function assertInRange(value: number, min: number, max: number, message: string): void;
declare const __e2eManifestDir: string;
declare function assertRejectsContains(
    action: () => Promise<unknown> | unknown,
    expectedSubstring: string,
    message: string,
): Promise<void>;
