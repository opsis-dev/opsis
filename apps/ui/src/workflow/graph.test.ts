import { describe, expect, it } from "vitest";
import type { Workflow } from "@/generated/bindings";
import { connects, portOf } from "./graph";

const node = (id: string, type: string) => ({ id, type, position: { x: 0, y: 0 }, params: {} });

const workflow = {
  version: 1,
  id: "wf",
  name: "Test",
  nodes: [
    node("title", "fx.core.title"),
    node("seq", "timeline.sequence"),
    node("file", "media.import"),
  ],
  edges: [],
  markers: [],
} as unknown as Workflow;

describe("growing inputs", () => {
  it("knows any numbered input of a node that grows, and nothing else", () => {
    expect(portOf(workflow, "seq", "in", "in12")?.type).toBe("clip");
    expect(portOf(workflow, "seq", "in", "in0")).toBeUndefined();
    expect(portOf(workflow, "seq", "in", "in1x")).toBeUndefined();
    expect(portOf(workflow, "title", "in", "in3")).toBeUndefined();
  });

  it("lets a clip connect to the ninth input of a sequence but not a file", () => {
    const ends = { sourceHandle: "clip", target: "seq", targetHandle: "in9" };
    expect(connects(workflow, { ...ends, source: "title" })).toBe(true);
    expect(connects(workflow, { ...ends, source: "file", sourceHandle: "media" })).toBe(false);
  });
});
