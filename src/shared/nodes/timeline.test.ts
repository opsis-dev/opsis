import { describe, expect, it } from "vitest";
import { emptyClip } from "../compile/clip";
import type { ClipValue } from "../schema/values";
import { DEFAULT_WORKFLOW_SETTINGS as settings, type Workflow } from "../schema/workflow";
import { must } from "../test-fixtures";
import { defaultParams, getNodeDef } from "./registry";

const workflow: Workflow = {
  version: 1,
  id: "wf",
  name: "Test",
  settings,
  variables: {},
  nodes: [],
  edges: [],
  markers: [],
};

function compileOver(type: string, params: Record<string, unknown>): ClipValue {
  const def = must(getNodeDef(type), type);
  const compile = must(def.compile, `compile for ${type}`);
  const out = compile({
    nodeId: "n",
    params: def.params.parse({ ...defaultParams(type), ...params }),
    inputs: { clip: emptyClip(90) },
    settings,
    vars: {},
    workflow,
  });
  const clip = out?.clip;
  if (clip?.type !== "clip") throw new Error(`no clip from ${type}`);
  return clip;
}

const startOf = (clip: ClipValue) => clip.tracks[0].items[0].from;

describe("timeline node lengths", () => {
  it("rounds a delay to whole frames, down to none for less than half a frame", () => {
    expect(startOf(compileOver("timeline.delay", { ms: 10 }))).toBe(0);
    expect(startOf(compileOver("timeline.delay", { ms: 0 }))).toBe(0);
    expect(startOf(compileOver("timeline.delay", { ms: 500 }))).toBe(15);
  });

  it("rounds a trim start to whole frames, down to none for less than half a frame", () => {
    const startFrom = (startMs: number) =>
      compileOver("timeline.trim", { startMs }).tracks[0].items[0].props.startFrom;
    expect(startFrom(10)).toBe(0);
    expect(startFrom(1000)).toBe(30);
  });
});
