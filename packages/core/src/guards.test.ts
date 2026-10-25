import { describe, expect, it } from "vitest";
import { z } from "zod";
import { parseJsonWith } from "#core/guards";

describe("parseJsonWith", () => {
  const schema = z.object({ a: z.number() });

  it("returns null when the text is not JSON", () => {
    expect(parseJsonWith(schema, "{a:")).toBeNull();
  });

  it("checks parsed JSON against the schema", () => {
    expect(parseJsonWith(schema, '{"a":1}')?.data).toEqual({ a: 1 });
    expect(parseJsonWith(schema, '{"a":"x"}')?.success).toBe(false);
  });
});
