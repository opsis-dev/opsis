import { describe, expect, it } from "vitest";
import { must } from "../test-fixtures";
import { parseParams } from "./params";
import { getNodeDef } from "./registry";

const problem = (type: string, params: Record<string, unknown>) => {
  const parsed = parseParams(must(getNodeDef(type), type).params, params);
  return parsed.ok ? null : parsed.problem;
};

describe("Crop", () => {
  it("keeps the area inside the recording and big enough to see", () => {
    expect(problem("camera.crop", { x: 0, y: 0, w: 0.5, h: 0.25 })).toBeNull();
    expect(problem("camera.crop", { x: 0.8, y: 0, w: 0.5, h: 0.5 })).toBe(
      "The area runs past the right edge",
    );
    expect(problem("camera.crop", { x: 0, y: 0.6, w: 0.5, h: 0.5 })).toBe(
      "The area runs past the bottom edge",
    );
    expect(problem("camera.crop", { x: 0, y: 0, w: 0, h: 0.5 })).toBe("Width is too small");
  });
});
