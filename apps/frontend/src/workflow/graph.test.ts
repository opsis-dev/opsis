import { describe, expect, it } from "vitest";
import type { Workflow, WorkflowEdge } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { connects, inputsBeforeCompile, portOf } from "./graph";

function node(id: string, type: string) {
  return { id, type, position: { x: 0, y: 0 }, params: {} };
}

const workflow = {
  version: 1,
  id: "wf",
  name: "Test",
  nodes: [
    node("title", "fx.title"),
    node("seq", "timeline.sequence"),
    node("file", "media.import"),
    node("take", "capture.record"),
    node("grade", "media.applyGrade"),
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

  it("refuses a take its current value can't turn into a file", () => {
    const ends = {
      source: "take",
      sourceHandle: "recording",
      target: "grade",
      targetHandle: "media",
    };
    expect(connects(workflow, ends)).toBe(true);
    expect(connects(workflow, ends, { take: { recording: ["recording"] } })).toBe(false);
    expect(connects(workflow, ends, { take: { recording: ["recording", "media"] } })).toBe(true);
  });
});

describe("inputs before the first compile", () => {
  it("shows every connected numbered input and one empty one after the last", () => {
    const def = getNodeDef("timeline.sequence");
    if (!def) throw new Error("no sequence");
    function edge(handle: string): WorkflowEdge {
      return {
        id: handle,
        source: "a",
        sourceHandle: "clip",
        target: "seq",
        targetHandle: handle,
      } as WorkflowEdge;
    }
    expect(Object.keys(inputsBeforeCompile(def, [], "seq"))).toEqual(Object.keys(def.inputs));
    const shown = inputsBeforeCompile(def, [edge("in1"), edge("in3")], "seq");
    expect(Object.keys(shown).filter((k) => k.startsWith("in"))).toEqual([
      "in1",
      "in2",
      "in3",
      "in4",
    ]);
    expect(shown.in4?.type).toBe("clip");
    expect(inputsBeforeCompile(def, [edge("in3")], "other")).toEqual(def.inputs);
  });
});
