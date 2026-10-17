import { compileWorkflow, previewSpec } from "./compile/compile";
import type { ActionLog } from "./schema/actions";
import type { NodeResult } from "./schema/values";
import type { Workflow } from "./schema/workflow";
import { GraphBuilder } from "./templates/build";
import { flattenSpec, type FlatTimeline } from "./timeline/flatten";

export const recordResult = (log: ActionLog, hash = "h"): NodeResult => ({
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
  artifacts: [],
  finishedAt: 0,
  durationMs: 0,
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

export function flatFilm(wf: Workflow): FlatTimeline {
  const target = previewSpec(wf.settings, compileWorkflow(wf), null);
  if (!target) throw new Error("nothing to preview");
  return flattenSpec(target.spec);
}
