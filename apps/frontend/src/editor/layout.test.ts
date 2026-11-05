import ElkConstructor from "elkjs/lib/elk.bundled.js";
import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import type { Workflow, WorkflowNode } from "@/generated/bindings";
import { autoLayout, type CardShape } from "./layout";

const elk = new ElkConstructor();

function node(id: string, type: string): WorkflowNode {
  return { id, type, position: { x: 0, y: 0 }, params: {} } as unknown as WorkflowNode;
}

function edge(source: string, target: string, targetHandle: string) {
  return {
    id: `${source}->${target}:${targetHandle}`,
    source,
    sourceHandle: "clip",
    target,
    targetHandle,
  };
}

function workflow(nodes: WorkflowNode[], edges: ReturnType<typeof edge>[]) {
  return { version: 1, id: "wf", name: "Test", nodes, edges, markers: [] } as unknown as Workflow;
}

function unmeasured(): CardShape {
  return { width: 256, height: 160, ports: null };
}

describe("tidy up layout", () => {
  it("lays out a sequence whatever numbered inputs its clips use", async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.uniqueArray(fc.integer({ min: 1, max: 9 }), { minLength: 1, maxLength: 5 }),
        async (inputs) => {
          const titles = inputs.map((i) => node(`t${i}`, "fx.title"));
          const wf = workflow(
            [...titles, node("seq", "timeline.sequence")],
            inputs.map((i) => edge(`t${i}`, "seq", `in${i}`)),
          );
          const placed = await autoLayout(elk, wf, unmeasured);
          expect(Object.keys(placed).toSorted()).toEqual(wf.nodes.map((n) => n.id).toSorted());
        },
      ),
      { numRuns: 15 },
    );
  });

  it("lines a card up with the handle its edge enters", async () => {
    const wf = workflow(
      [node("title", "fx.title"), node("seq", "timeline.sequence")],
      [edge("title", "seq", "in2")],
    );
    const shapes: Record<string, CardShape> = {
      title: { width: 256, height: 140, ports: [{ handle: "clip", side: "out", y: 112 }] },
      seq: {
        width: 256,
        height: 220,
        ports: [
          { handle: "in1", side: "in", y: 150 },
          { handle: "in2", side: "in", y: 174 },
          { handle: "in3", side: "in", y: 198 },
          { handle: "clip", side: "out", y: 150 },
        ],
      },
    };
    const placed = await autoLayout(elk, wf, (n) => shapes[n.id] ?? unmeasured());
    expect(placed.title.y + 112).toBeCloseTo(placed.seq.y + 174, 0);
  });
});
