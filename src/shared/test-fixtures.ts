import { walkItems } from "./compile/clip";
import { compileWorkflow, previewSpec } from "./compile/compile";
import type { CardContext, CardRow } from "./nodes/card";
import { cardRows, type CompileInputs, type CompileOutputs } from "./nodes/define";
import { defaultParams, getNodeDef } from "./nodes/registry";
import type { Action, ActionLog } from "./schema/actions";
import type { TimelineItem, TimelineSpec } from "./schema/timeline";
import type { ClipValue, NodeResult } from "./schema/values";
import { DEFAULT_WORKFLOW_SETTINGS, type Workflow, type WorkflowNode } from "./schema/workflow";
import { GraphBuilder, type TakeStep } from "./templates/build";
import { flattenSpec, type FlatTimeline } from "./timeline/flatten";

export const TITLE = "fx.core.title";

export const workflowNode = (
  type: string,
  params: Record<string, unknown> = {},
  id = "n",
): WorkflowNode => ({ id, type, position: { x: 0, y: 0 }, params });

export const makeLog = (actions: Action[], durationMs: number): ActionLog => ({
  viewport: { width: 1920, height: 1080 },
  dpr: 1,
  t0: 0,
  durationMs,
  actions,
  cursorTrack: [{ t: 0, x: 960, y: 540 }],
});

export const nodeResult = (fields: Partial<NodeResult> = {}): NodeResult => ({
  hash: "h",
  outputs: {},
  artifacts: [],
  finishedAt: 0,
  durationMs: 0,
  ...fields,
});

export const recordResult = (log: ActionLog, hash = "h"): NodeResult =>
  nodeResult({
    hash,
    outputs: {
      recording: {
        type: "recording",
        src: `project:.demoflow/cache/${hash}/recording.mp4`,
        log,
        durationMs: log.durationMs,
        viewport: log.viewport,
        nodeId: "record",
      },
    },
  });

export const recordedTake = (g: GraphBuilder, steps: TakeStep[] = []): string =>
  g.recordTake(
    [
      { id: "session", type: "browser.session" },
      { id: "open", type: "browser.navigate", params: { url: "https://example.com" } },
      ...steps,
    ],
    { camera: { speed: 1 } },
  );

export function buildFilm(fn: (g: GraphBuilder) => string): Workflow {
  const g = new GraphBuilder();
  const last = fn(g);
  g.node("out", "output.render", 9, 0);
  g.link(last, "out", "clip", "timeline");
  return g.build({ id: "wf", name: "Test" });
}

export function must<T>(value: T | null | undefined, what: string): T {
  if (value === null || value === undefined) throw new Error(`no ${what}`);
  return value;
}

export const filmSpec = (wf: Workflow): TimelineSpec =>
  must(previewSpec(wf.settings, compileWorkflow(wf), null), "film to preview").spec;

export const flatFilm = (wf: Workflow): FlatTimeline => flattenSpec(filmSpec(wf));

export function itemsOf(spec: Pick<TimelineSpec, "tracks">): TimelineItem[] {
  const items: TimelineItem[] = [];
  walkItems(spec.tracks, (item) => items.push(item));
  return items;
}

const emptyWorkflow = (): Workflow => ({
  version: 1,
  id: "wf",
  name: "Test",
  settings: DEFAULT_WORKFLOW_SETTINGS,
  variables: {},
  nodes: [],
  edges: [],
  markers: [],
});

export function nodeCard(
  type: string,
  params: Record<string, unknown> = {},
  extra: Partial<CardContext<Record<string, unknown>>> = {},
): CardRow[] {
  const def = must(getNodeDef(type), type);
  return cardRows(def, {
    params: { ...defaultParams(type), ...params },
    settings: DEFAULT_WORKFLOW_SETTINGS,
    connected: [],
    ...extra,
  });
}

export function compileNode(
  type: string,
  params: Record<string, unknown>,
  inputs: CompileInputs = {},
): CompileOutputs {
  const def = must(getNodeDef(type), type);
  const compile = must(def.compile, `compile for ${type}`);
  const workflow = emptyWorkflow();
  return (
    compile({
      nodeId: "n",
      params: def.params.parse({ ...defaultParams(type), ...params }),
      inputs,
      settings: workflow.settings,
      vars: {},
      workflow,
    }) ?? {}
  );
}

export function compileClip(
  type: string,
  params: Record<string, unknown>,
  inputs: CompileInputs = {},
): ClipValue {
  const clip = compileNode(type, params, inputs).clip;
  if (clip?.type !== "clip") throw new Error(`no clip from ${type}`);
  return clip;
}

type Modifiers = Partial<Record<"ctrlKey" | "metaKey" | "altKey" | "shiftKey", boolean>>;

export const keyEvent = (key: string, mods: Modifiers = {}) => ({
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});
