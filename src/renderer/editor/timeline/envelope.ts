import { getNodeDef } from "@/shared/nodes/registry";
import {
  audioEnvelopeSchema,
  visualEnvelopeSchema,
  type AudioEnvelopeKey,
  type Keyframe,
  type VisualEnvelopeKey,
} from "@/shared/schema/timeline";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";
import { envelopeAt, sortKeys } from "@/shared/timeline/envelope";
import { useWorkflowStore } from "../../store/workflow";

export type EnvelopeKey = VisualEnvelopeKey | AudioEnvelopeKey;

type EnvelopeInfo = { label: string; min: number; max: number; rest: number };

export const ENVELOPE_INFO: Record<EnvelopeKey, EnvelopeInfo> = {
  opacity: { label: "Opacity", min: 0, max: 1, rest: 1 },
  x: { label: "Move across", min: -1, max: 1, rest: 0 },
  y: { label: "Move up and down", min: -1, max: 1, rest: 0 },
  scale: { label: "Size", min: 0, max: 3, rest: 1 },
  volume: { label: "Volume", min: 0, max: 2, rest: 1 },
  pan: { label: "Balance", min: -1, max: 1, rest: 0 },
};

const envelopeSchema = visualEnvelopeSchema.extend(audioEnvelopeSchema.shape);

type Envelope = Partial<Record<EnvelopeKey, Keyframe[]>>;

function readEnvelope(node: WorkflowNode): Envelope {
  const parsed = envelopeSchema.safeParse(node.params.envelope);
  return parsed.success ? parsed.data : {};
}

export function envelopeKeysFor(
  wf: Workflow,
  nodeId: string,
  prop: EnvelopeKey,
): Keyframe[] | null {
  const node = wf.nodes.find((n) => n.id === nodeId);
  if (!node || !getNodeDef(node.type)?.params.shape.envelope) return null;
  return readEnvelope(node)[prop] ?? [];
}

export function setEnvelopeKeys(nodeId: string, prop: EnvelopeKey, keys: Keyframe[]): void {
  useWorkflowStore.getState().update((wf) => ({
    ...wf,
    nodes: wf.nodes.map((n) => {
      if (n.id !== nodeId) return n;
      const envelope = readEnvelope(n);
      if (keys.length) envelope[prop] = sortKeys(keys);
      else delete envelope[prop];
      return { ...n, params: { ...n.params, envelope } };
    }),
  }));
}

export function addKeyframe(nodeId: string, prop: EnvelopeKey, keys: Keyframe[], at: number) {
  const frame = Math.max(0, Math.round(at));
  setEnvelopeKeys(nodeId, prop, [
    ...keys,
    { frame, value: envelopeAt(keys, frame, ENVELOPE_INFO[prop].rest), ease: "linear" },
  ]);
}
