import type { Viewport, Workflow, WorkflowEdge, WorkflowSettings } from "@/generated/bindings";

export type EdgeEnds = Omit<WorkflowEdge, "id">;

export const THEME_BACKGROUNDS: Record<WorkflowSettings["theme"], string> = {
  light: "#fafafa",
  dark: "#09090b",
};

export const DEFAULT_VIEWPORT: Viewport = { width: 1920, height: 1080 };

export function normalizeWorkflow(wf: Workflow): Workflow {
  if (!wf.mix) return wf;
  const ids = new Set(wf.nodes.map((n) => n.id));
  const channels = Object.entries(wf.mix.channels).filter(([id]) => ids.has(id));
  return { ...wf, mix: { ...wf.mix, channels: Object.fromEntries(channels) } };
}
