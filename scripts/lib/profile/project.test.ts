import { describe, expect, it } from "vitest";
import { editorCatalog } from "../tools";
import {
  type CatalogNode,
  missingTypes,
  profileWorkflow,
  runOrder,
  type Workflow,
} from "./project";

type Check = { rule: string };

type CheckedNode = CatalogNode & {
  params?: { properties?: Record<string, { "x-check"?: Check | Check[] }> };
};

type FullCatalog = {
  connects: { from: string; to: string }[];
  nodes: CheckedNode[];
};

const catalog = editorCatalog() as FullCatalog;
const workflow: Workflow = profileWorkflow(catalog, "http://127.0.0.1:4321");

describe("the profile project", () => {
  it("holds every node type in the catalogue", () => {
    expect(missingTypes(catalog, workflow)).toEqual([]);
  });

  it("joins only ports that exist and whose kinds connect", () => {
    const definitions = new Map(catalog.nodes.map((node) => [node.type, node]));
    const types = new Map(workflow.nodes.map((node) => [node.id, node.type]));
    const connects = new Set(catalog.connects.map((rule) => `${rule.from}>${rule.to}`));
    const wrong = workflow.edges.filter((edge) => {
      const from = definitions.get(types.get(edge.source) ?? "")?.outputs[edge.sourceHandle];
      const to = definitions.get(types.get(edge.target) ?? "")?.inputs[edge.targetHandle];
      return !from || !to || !connects.has(`${from.type}>${to.type}`);
    });
    expect(wrong.map((edge) => edge.id)).toEqual([]);
  });

  it("fills every setting that may not be blank", () => {
    const definitions = new Map(catalog.nodes.map((node) => [node.type, node]));
    const blank = workflow.nodes.flatMap((node) =>
      Object.entries(definitions.get(node.type)?.params?.properties ?? {})
        .filter(([, schema]) =>
          [schema["x-check"] ?? []].flat().some((check) => check.rule === "not-blank"),
        )
        .filter(([name]) => String(node.params[name] ?? "").trim() === "")
        .map(([name]) => `${node.id}.${name}`),
    );
    expect(blank).toEqual([]);
  });

  it("runs every working node once, after the working nodes it reads from", () => {
    const order = runOrder(catalog, workflow);
    const executes = new Set(
      catalog.nodes.filter((node) => node.executes).map((node) => node.type),
    );
    const working = workflow.nodes.filter((node) => executes.has(node.type)).map((node) => node.id);
    expect([...order].sort()).toEqual([...working].sort());
    const position = new Map(order.map((id, index) => [id, index]));
    const backwards = workflow.edges.filter((edge) => {
      const source = position.get(edge.source);
      const target = position.get(edge.target);
      return source !== undefined && target !== undefined && source >= target;
    });
    expect(backwards.map((edge) => edge.id)).toEqual([]);
  });
});
