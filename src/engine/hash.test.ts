import { describe, expect, it } from "vitest";
import { versionedHash } from "./hash";

describe("versionedHash", () => {
  it("ignores key order and changes with the version", () => {
    expect(versionedHash("v1", { a: 1, b: 2 })).toBe(versionedHash("v1", { b: 2, a: 1 }));
    expect(versionedHash("v1", { a: 1 })).not.toBe(versionedHash("v2", { a: 1 }));
    expect(versionedHash("v1", { a: 1 })).toHaveLength(24);
  });
});
