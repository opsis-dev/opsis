import { describe, expect, it } from "vitest";
import { getTemplate } from "../templates";
import { must } from "../test-fixtures";
import { checkWorkflow } from "./check";

const todo = () => getTemplate("todomvc").build("Check");

describe("checkWorkflow", () => {
  it("accepts a template project", () => {
    const result = checkWorkflow(todo());
    expect(result.ok).toBe(true);
    expect(result.workflow?.nodes.length).toBeGreaterThan(5);
  });

  it("reports format problems with a path", () => {
    const result = checkWorkflow({
      version: 1,
      id: "x",
      name: "x",
      nodes: [{ id: "a" }],
      edges: [],
    });
    expect(result.ok).toBe(false);
    expect(result.workflow).toBeNull();
    expect(result.problems.map((p) => p.path)).toContain("nodes[0].type");
  });

  it("reports unknown node types, bad connections and duplicate ids", () => {
    const wf = todo();
    const first = wf.nodes[0];
    wf.nodes.push({ id: "ghost", type: "nope.nothing", position: { x: 0, y: 0 }, params: {} });
    wf.nodes.push({ ...first });
    wf.edges.push({
      id: "bad",
      source: first.id,
      sourceHandle: "no-such-port",
      target: "ghost",
      targetHandle: "in",
    });
    const result = checkWorkflow(wf);
    expect(result.ok).toBe(false);
    const messages = result.problems.map((p) => p.message);
    expect(messages).toContain("This node type isn't available");
    expect(messages).toContain("Two nodes share this id");
    expect(result.problems.some((p) => p.edgeId === "bad")).toBe(true);
  });

  it("reports a setting that isn't valid", () => {
    const wf = todo();
    const node = must(
      wf.nodes.find((n) => n.type === "browser.navigate"),
      "navigate node",
    );
    node.params = { ...node.params, url: 42 };
    const result = checkWorkflow(wf);
    expect(result.problems).toContainEqual({
      level: "error",
      nodeId: node.id,
      message: "Page address isn't valid",
    });
  });
});
