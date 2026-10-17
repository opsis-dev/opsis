import path from "node:path";
import { describe, expect, it } from "vitest";
import { isInside } from "./project-paths";

describe("isInside", () => {
  const root = path.resolve("/projects/demo");

  it("accepts the folder and anything under it, including names that start with dots", () => {
    expect(isInside(root, root)).toBe(true);
    expect(isInside(root, path.join(root, "assets", "clip.mp4"))).toBe(true);
    expect(isInside(root, path.join(root, "..notes.png"))).toBe(true);
  });

  it("refuses anything outside", () => {
    expect(isInside(root, path.resolve(root, ".."))).toBe(false);
    expect(isInside(root, path.resolve(root, "..", "other", "a.png"))).toBe(false);
    expect(isInside(root, path.resolve("/elsewhere/a.png"))).toBe(false);
  });
});
