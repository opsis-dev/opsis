import { z } from "zod";
import { describe, expect, it } from "vitest";
import { parseParams, target } from "./params";

const schema = z.object({
  name: z.string().meta({ description: "Name" }),
  count: z.number().min(1).max(3),
  target: target(),
});

const problem = (values: unknown) => {
  const parsed = parseParams(schema, values);
  return parsed.ok ? null : parsed.problem;
};

describe("parseParams", () => {
  const valid = { name: "a", count: 2, target: "Save" };

  it("returns the parsed settings", () => {
    expect(parseParams(schema, valid)).toEqual({ ok: true, data: valid });
  });

  it("names the first setting that is wrong and how", () => {
    expect(problem({ ...valid, name: undefined })).toBe("Name is missing");
    expect(problem({ ...valid, name: 3 })).toBe("Name isn't valid");
    expect(problem({ ...valid, count: 0 })).toBe("Count is too small");
    expect(problem({ ...valid, count: 9 })).toBe("Count is too large");
    expect(problem({ ...valid, target: "" })).toBe("Element is empty");
  });
});
