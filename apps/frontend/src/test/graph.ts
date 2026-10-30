import type {
  LocalFrames,
  FlatTimeline,
  NodeResult,
  TimelineSpec,
  Workflow,
  WorkflowEdge,
  WorkflowNode,
  WorkflowSettings,
} from "@/generated/bindings";
import { defaultParams } from "@/store/catalog";
import { edgeId, randomId } from "@/workflow/ids";
import { CLIP_PORT, COLUMN_WIDTH, ROW_HEIGHT } from "@/workflow/nodes";
import { editorData } from "./editor-data";

export const TITLE = "fx.title";

const SETTINGS: WorkflowSettings = {
  width: 1920,
  height: 1080,
  fps: 30,
  theme: "dark",
  fonts: { main: "Inter", heading: "", code: "JetBrains Mono", families: [] },
};

export class GraphBuilder {
  nodes: WorkflowNode[] = [];
  edges: WorkflowEdge[] = [];

  node(
    id: string,
    type: string,
    x: number,
    y: number,
    params: Record<string, unknown> = {},
    label?: string,
  ): string {
    this.nodes.push({
      id,
      type,
      position: { x: x * COLUMN_WIDTH, y: y * ROW_HEIGHT },
      params: { ...defaultParams(type), ...params },
      ...(label ? { label } : {}),
    });
    return id;
  }

  link(source: string, target: string, sourceHandle = CLIP_PORT, targetHandle = CLIP_PORT): this {
    const ends = { source, sourceHandle, target, targetHandle };
    this.edges.push({ id: edgeId(ends), ...ends });
    return this;
  }

  build(meta: { id?: string; name: string; settings?: Partial<WorkflowSettings> }): Workflow {
    return {
      version: 1,
      id: meta.id ?? randomId("wf"),
      name: meta.name,
      settings: { ...SETTINGS, ...meta.settings },
      variables: {},
      nodes: [...this.nodes],
      edges: [...this.edges],
      markers: [],
    };
  }
}

export function workflowNode(
  type: string,
  params: Record<string, unknown> = {},
  id = "n",
): WorkflowNode {
  return { id, type, position: { x: 0, y: 0 }, params };
}

export function nodeResult(fields: Partial<NodeResult> = {}): NodeResult {
  return {
    hash: "h",
    outputs: {},
    artifacts: [],
    finishedAt: 0,
    durationMs: 0,
    ...fields,
  };
}

export function must<T>(value: T | null | undefined, what: string): T {
  if (value === null || value === undefined) throw new Error(`no ${what}`);
  return value;
}

export function buildFilm(fn: (g: GraphBuilder) => string): Workflow {
  const g = new GraphBuilder();
  const last = fn(g);
  g.node("out", "output.render", 9, 0);
  g.link(last, "out");
  return g.build({ id: "wf", name: "Test" });
}

type Film = {
  spec: TimelineSpec | null;
  flat: FlatTimeline | null;
  localFrames: Record<string, LocalFrames>;
};

export function film(wf: Workflow): Film {
  return editorData("film", JSON.stringify(wf)) as Film;
}

export function flatFilm(wf: Workflow): FlatTimeline {
  return must(film(wf).flat, "film to flatten");
}
