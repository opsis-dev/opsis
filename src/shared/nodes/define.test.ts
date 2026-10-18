import { describe, expect, it } from "vitest";
import { z } from "zod";
import { defineNode, parseNodeParams } from "./define";

const exportNode = defineNode({
  type: "test.export",
  title: "Test export",
  category: "Output",
  phase: "render",
  inputs: {},
  outputs: {},
  params: z.object({
    fileName: z.string().default("{{Project name}}"),
    atMs: z.number().default(1000),
  }),
});

describe("parseNodeParams", () => {
  it("fills a missing setting from its default before replacing variables", () => {
    const params = parseNodeParams(
      exportNode,
      { params: { atMs: 500 } },
      { "Project name": "Demo" },
    );
    expect(params.fileName).toBe("Demo");
    expect(params.atMs).toBe(500);
  });

  it("keeps a stored setting over the default", () => {
    const params = parseNodeParams(
      exportNode,
      { params: { fileName: "{{Project name}} still" } },
      { "Project name": "Demo" },
    );
    expect(params.fileName).toBe("Demo still");
  });
});
