import path from "node:path";
import { describe, expect, it } from "vitest";
import { parseRange, resolveInside } from "./http-file";

describe("parseRange", () => {
  it("reads open, closed and suffix ranges", () => {
    expect(parseRange("bytes=0-", 100)).toEqual({ start: 0, end: 99 });
    expect(parseRange("bytes=10-19", 100)).toEqual({ start: 10, end: 19 });
    expect(parseRange("bytes=10-500", 100)).toEqual({ start: 10, end: 99 });
    expect(parseRange("bytes=-30", 100)).toEqual({ start: 70, end: 99 });
  });

  it("refuses ranges outside the file", () => {
    expect(parseRange("bytes=-", 100)).toBe("unsatisfiable");
    expect(parseRange("bytes=100-", 100)).toBe("unsatisfiable");
    expect(parseRange("bytes=50-10", 100)).toBe("unsatisfiable");
    expect(parseRange("bytes=-0", 100)).toBe("unsatisfiable");
    expect(parseRange("bytes=0-", 0)).toBe("unsatisfiable");
  });

  it("ignores a missing header", () => {
    expect(parseRange(null, 100)).toBeNull();
    expect(parseRange(undefined, 100)).toBeNull();
  });
});

describe("resolveInside", () => {
  const root = path.resolve("/srv/project");

  it("keeps files inside the root", () => {
    expect(resolveInside(root, "/assets/a%20b.mp4")).toBe(path.join(root, "assets", "a b.mp4"));
    expect(resolveInside(root, "/")).toBe(root);
  });

  it("refuses paths that leave the root", () => {
    expect(resolveInside(root, "/../secret.txt")).toBeNull();
    expect(resolveInside(root, "/%2e%2e/secret.txt")).toBeNull();
  });

  it("refuses paths with broken escapes", () => {
    expect(resolveInside(root, "/assets/%E0%A4%A.mp4")).toBeNull();
  });
});
