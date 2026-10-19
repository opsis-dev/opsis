import { describe, expect, it } from "vitest";
import type { FontFamilyDef } from "./fonts/types";
import { findById } from "./lists";
import { DEFAULT_STRIP, defaultMix } from "./mix/types";
import { GROUP_TYPE } from "./nodes/utility";
import { GraphBuilder } from "./templates/build";
import { must, workflowNode } from "./test-fixtures";
import {
  addProjectFont,
  connect,
  GROUP_PAD,
  GROUP_TITLE_HEIGHT,
  groupNodes,
  pasteNodes,
  removeNodes,
  renameWorkflowVariable,
  ungroup,
} from "./workflow-edits";

const SIZE = { width: 200, height: 100 };

function sounds() {
  const g = new GraphBuilder();
  g.node("a", "audio.sfx", 0, 0);
  g.node("b", "audio.sfx", 0, 1);
  g.node("mix", "audio.mix", 1, 0);
  g.node("m", "audio.music", 0, 2);
  return g.build({ id: "wf", name: "Sounds" });
}

const grouped = () =>
  groupNodes(sounds(), ["a", "b"], { id: "g", title: "Effects", sizeOf: () => SIZE });

describe("workflow edits", () => {
  it("lets a many port take several sources but replaces a single port's source", () => {
    const one = connect(sounds(), {
      source: "a",
      sourceHandle: "audio",
      target: "mix",
      targetHandle: "sounds",
    });
    const two = connect(one, {
      source: "b",
      sourceHandle: "audio",
      target: "mix",
      targetHandle: "sounds",
    });
    expect(two.edges.map((e) => e.id)).toEqual(["a.audio->mix.sounds", "b.audio->mix.sounds"]);
    const clipA = connect(two, {
      source: "a",
      sourceHandle: "x",
      target: "mix",
      targetHandle: "clip",
    });
    const clipB = connect(clipA, {
      source: "b",
      sourceHandle: "x",
      target: "mix",
      targetHandle: "clip",
    });
    expect(clipB.edges.filter((e) => e.targetHandle === "clip").map((e) => e.source)).toEqual([
      "b",
    ]);
  });

  it("groups around the members and ungroups back to the same places", () => {
    const wf = grouped();
    const group = findById(wf.nodes, "g");
    expect(group?.type).toBe(GROUP_TYPE);
    const [a, b] = sounds().nodes;
    const top = GROUP_PAD + GROUP_TITLE_HEIGHT;
    expect(group?.position).toEqual({ x: a.position.x - GROUP_PAD, y: a.position.y - top });
    expect(group?.size).toEqual({
      width: SIZE.width + 2 * GROUP_PAD,
      height: b.position.y - a.position.y + SIZE.height + top + GROUP_PAD,
    });
    expect(wf.nodes.filter((n) => n.parentId === "g").map((n) => n.id)).toEqual(["a", "b"]);
    expect(ungroup(wf, "g").nodes).toEqual(sounds().nodes);
  });

  it("keeps a group of fewer than two free nodes from forming", () => {
    const wf = sounds();
    expect(groupNodes(wf, ["a"], { id: "g", title: "x", sizeOf: () => SIZE })).toBe(wf);
  });

  it("removes nodes with their edges and mixer strips and frees the children of a removed group", () => {
    const linked = connect(grouped(), {
      source: "m",
      sourceHandle: "audio",
      target: "mix",
      targetHandle: "sounds",
    });
    const mix = defaultMix();
    mix.channels.m = { ...DEFAULT_STRIP, fader: -6 };
    mix.channels.a = { ...DEFAULT_STRIP, fader: -3 };
    const wf = removeNodes({ ...linked, mix }, ["g", "m"]);
    expect(wf.nodes.map((n) => n.id)).toEqual(["a", "b", "mix"]);
    expect(findById(wf.nodes, "a")).toEqual(sounds().nodes[0]);
    expect(wf.edges).toEqual([]);
    expect(Object.keys(wf.mix?.channels ?? {})).toEqual(["a"]);
  });

  it("pastes copies with new ids, offset outside groups, and keeps their edges", () => {
    const source = connect(grouped(), {
      source: "a",
      sourceHandle: "audio",
      target: "mix",
      targetHandle: "sounds",
    });
    const copied = source.nodes.filter((n) => ["g", "a", "mix"].includes(n.id));
    const { workflow, ids } = pasteNodes(source, copied, source.edges, { x: 10, y: 20 });
    expect(ids).toHaveLength(3);
    const pasted = workflow.nodes.slice(source.nodes.length);
    const [group, a, mixer] = pasted;
    expect(group.position).toEqual({ x: copied[0].position.x + 10, y: copied[0].position.y + 20 });
    expect(a.parentId).toBe(group.id);
    expect(a.position).toEqual(copied[1].position);
    expect(workflow.edges.at(-1)).toMatchObject({
      id: `${a.id}.audio->${mixer.id}.sounds`,
      source: a.id,
      target: mixer.id,
    });
  });

  it("pastes a grouped node without its group at its place on the canvas", () => {
    const source = grouped();
    const group = must(findById(source.nodes, "g"), "group");
    const inGroup = must(findById(source.nodes, "a"), "grouped node");
    const { workflow } = pasteNodes(source, [inGroup], [], { x: 10, y: 20 });
    const pasted = workflow.nodes.at(-1);
    expect(pasted && "parentId" in pasted).toBe(false);
    expect(pasted?.position).toEqual({
      x: group.position.x + inGroup.position.x + 10,
      y: group.position.y + inGroup.position.y + 20,
    });
  });

  it("renames a variable everywhere unless the new name is blank", () => {
    const wf = {
      ...sounds(),
      variables: { site: "https://example.com" },
      nodes: [workflowNode("browser.navigate", { url: "{{site}}" })],
    };
    const renamed = renameWorkflowVariable(wf, "site", "home");
    expect(renamed.variables).toEqual({ home: "https://example.com" });
    expect(renamed.nodes[0].params.url).toBe("{{home}}");
    expect(renameWorkflowVariable(wf, "site", " ")).toBe(wf);
    expect(renameWorkflowVariable(wf, "site", "")).toBe(wf);
    expect(
      renameWorkflowVariable({ ...wf, variables: { site: "a", home: "b" } }, "site", "home")
        .variables,
    ).toEqual({ site: "a", home: "b" });
    expect(renameWorkflowVariable(wf, "missing", "other")).toBe(wf);
    expect(renameWorkflowVariable(wf, "constructor", "other")).toBe(wf);
    expect(renameWorkflowVariable(wf, "site", "constructor").variables).toEqual({
      constructor: "https://example.com",
    });
  });

  it("replaces a project font of the same family", () => {
    const font = (id: string): FontFamilyDef => ({
      id,
      family: "Inter",
      category: "sans-serif",
      source: "file",
      faces: [],
    });
    const once = addProjectFont(sounds(), font("one"));
    const twice = addProjectFont(once, font("two"));
    expect(twice.settings.fonts.families.filter((f) => f.family === "Inter")).toEqual([
      font("two"),
    ]);
  });
});
