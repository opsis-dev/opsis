import { walkItems } from "./compile/clip";
import { compileWorkflow, previewSpec } from "./compile/compile";
import type { ActionLog } from "./schema/actions";
import type { TimelineItem, TimelineSpec } from "./schema/timeline";
import type { NodeResult } from "./schema/values";
import type { Workflow } from "./schema/workflow";
import { GraphBuilder } from "./templates/build";
import { flattenSpec, type FlatTimeline } from "./timeline/flatten";

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

type TakeStep = { id: string; type: string; params: Record<string, unknown> };

export function recordedTake(g: GraphBuilder, steps: TakeStep[] = []): string {
  const session = g.node("session", "browser.session", 0, 0);
  const open = g.node("open", "browser.navigate", 1, 0, { url: "https://example.com" });
  const actions = steps.map((s, i) => g.node(s.id, s.type, 2 + i, 0, s.params));
  const record = g.node("record", "capture.record", 2 + steps.length, 0);
  g.chain([session, open, ...actions, record]);
  const camera = g.node("camera", "camera.autoZoom", 2 + steps.length, 1, { speed: 1 });
  g.link(record, camera, "recording", "recording");
  return camera;
}

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
