import { describe, expect, it } from "vitest";
import { z } from "zod";
import { fieldMeta } from "../elements/introspect";
import {
  choiceFrom,
  choiceOf,
  loudnessName,
  more,
  parseParams,
  qualityName,
  targetFields,
  text,
} from "./params";

const schema = z.object({
  name: z.string().meta({ description: "Name" }),
  count: z.number().min(1).max(3),
  ...targetFields(),
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

describe("loudness names", () => {
  it("names the presets and writes other levels with a real minus sign", () => {
    expect(loudnessName(-14)).toBe("Online video");
    expect(loudnessName(-18)).toBe("−18 LUFS");
  });
});

describe("field builders", () => {
  it("adds to a field's settings without losing the ones it had", () => {
    const field = more(text("", "Name", "textarea"), { help: "Who it's for", advanced: true });
    expect(fieldMeta(field)).toMatchObject({
      description: "Name",
      widget: "textarea",
      help: "Who it's for",
      advanced: true,
    });
  });

  it("labels choices from a table of words or entries with a label", () => {
    const field = choiceOf({ a: "First", b: { label: "Second" } }, "a", "Pick");
    expect(fieldMeta(field).labels).toEqual({ a: "First", b: "Second" });
    expect(field.parse(undefined)).toBe("a");
  });

  it("offers only the chosen part of a table, with its labels", () => {
    const field = choiceFrom({ x: "Ex", y: "Why", z: "Zed" }, ["x", "z"], "z", "Pick");
    expect(fieldMeta(field).labels).toEqual({ x: "Ex", z: "Zed" });
    expect(field.safeParse("y").success).toBe(false);
  });
});

describe("qualityName", () => {
  it("names the nearest preset and prefers the finer one on a tie", () => {
    expect(qualityName(18)).toBe("High");
    expect(qualityName(25)).toBe("Medium");
    expect(qualityName(15)).toBe("Max");
  });
});
