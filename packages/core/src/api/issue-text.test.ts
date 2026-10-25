import { describe, expect, it } from "vitest";
import { z } from "zod";
import { addEdgeBody, addNodeBody } from "./bodies";
import { firstIssueText } from "./issue-text";

describe("first issue text", () => {
  it("names the field the way the project check does", () => {
    const schema = z.object({ nodes: z.array(z.object({ "x y": z.number() })) });
    const parsed = schema.safeParse({ nodes: [{ "x y": "1" }] });
    if (parsed.success) throw new Error("Expected a bad value.");
    expect(firstIssueText(parsed.error)).toBe(
      'nodes[0]["x y"]: Invalid input: expected number, received string',
    );
  });

  it("gives the bare message when the whole value is wrong", () => {
    const parsed = z.number().safeParse("1");
    if (parsed.success) throw new Error("Expected a bad value.");
    expect(firstIssueText(parsed.error)).toBe("Invalid input: expected number, received string");
  });

  it("names each request body field once", () => {
    const text = (result: { success: boolean; error?: z.ZodError }) =>
      result.error ? firstIssueText(result.error) : "valid";
    expect(text(addNodeBody.safeParse({}))).toBe("type: is required.");
    expect(text(addNodeBody.safeParse({ type: "t", position: { x: "a", y: 1 } }))).toBe(
      "position.x: must be a number.",
    );
    expect(text(addEdgeBody.safeParse({ target: "a" }))).toBe("source: is required.");
  });
});
