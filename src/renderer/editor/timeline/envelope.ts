import { getNodeDef } from "@/shared/nodes/registry";
import { MAX_VOLUME } from "@/shared/nodes/params";
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
  volume: { label: "Volume", min: 0, max: MAX_VOLUME, rest: 1 },
  pan: { label: "Balance", min: -1, max: 1, rest: 0 },
};

const PAD = 5;

const span = (prop: EnvelopeKey) => ENVELOPE_INFO[prop].max - ENVELOPE_INFO[prop].min;

export const envelopeY = (prop: EnvelopeKey, value: number, height: number): number =>
  PAD + (1 - (value - ENVELOPE_INFO[prop].min) / span(prop)) * (height - PAD * 2);

export const envelopeShift = (prop: EnvelopeKey, dy: number, height: number): number =>
  (dy / Math.max(1, height - PAD * 2)) * span(prop);

export type EnvelopeTarget = { nodeId: string; prop: EnvelopeKey; keys: Keyframe[] };

const envelopeSchema = visualEnvelopeSchema.extend(audioEnvelopeSchema.shape);

type Envelope = Partial<Record<EnvelopeKey, Keyframe[]>>;

const parsed = new WeakMap<object, Envelope | null>();

function readEnvelope(node: WorkflowNode): Envelope | null {
  const raw = node.params.envelope;
  if (raw === undefined) return {};
  if (typeof raw !== "object" || raw === null) return null;
  if (!parsed.has(raw)) {
    const result = envelopeSchema.safeParse(raw);
    parsed.set(raw, result.success ? result.data : null);
  }
  return parsed.get(raw) ?? null;
}

export function envelopeTarget(
  wf: Workflow,
  nodeId: string | undefined,
  prop: EnvelopeKey | null,
): EnvelopeTarget | null {
  if (!nodeId || !prop) return null;
  const node = wf.nodes.find((n) => n.id === nodeId);
  if (!node || !getNodeDef(node.type)?.params.shape.envelope) return null;
  const envelope = readEnvelope(node);
  return envelope ? { nodeId, prop, keys: envelope[prop] ?? [] } : null;
}

export function setEnvelopeKeys(nodeId: string, prop: EnvelopeKey, keys: Keyframe[]): void {
  useWorkflowStore.getState().update((wf) => ({
    ...wf,
    nodes: wf.nodes.map((n) => {
      const current = n.id === nodeId ? readEnvelope(n) : null;
      if (!current) return n;
      const envelope = { ...current };
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
