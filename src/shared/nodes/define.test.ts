import { describe, expect, it } from "vitest";
import { parseNodeParams } from "./define";
import { stillOutputNode } from "./output";

describe("parseNodeParams", () => {
  it("fills a missing setting from its default before replacing variables", () => {
    const params = parseNodeParams(
      stillOutputNode,
      { params: { atMs: 500 } },
      { "Project name": "Demo" },
    );
    expect(params.fileName).toBe("Demo");
    expect(params.atMs).toBe(500);
  });

  it("keeps a stored setting over the default", () => {
    const params = parseNodeParams(
      stillOutputNode,
      { params: { fileName: "{{Project name}} still" } },
      { "Project name": "Demo" },
    );
    expect(params.fileName).toBe("Demo still");
  });
});
