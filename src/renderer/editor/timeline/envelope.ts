import { toast } from "sonner";
import { MAX_VOLUME } from "@/shared/limits";
import { findById } from "@/shared/lists";
import type { MinMax } from "@/shared/math";
import { parseOncePer } from "@/shared/memo";
import { getNodeDef } from "@/shared/nodes/registry";
import {
  envelopeSchema,
  type Envelope,
  type EnvelopeKey,
  type Keyframe,
} from "@/shared/schema/timeline";
import type { Workflow, WorkflowNode } from "@/shared/schema/workflow";
import { envelopeAt, sortKeys } from "@/shared/timeline/envelope";
import { wholeFrames } from "@/shared/timeline/time";
import { patchNodeParams } from "@/shared/workflow-edits";
import { useWorkflowStore } from "../../store/workflow";

type EnvelopeInfo = MinMax & { rest: number };

export const ENVELOPE_INFO: Record<EnvelopeKey, EnvelopeInfo> = {
  opacity: { min: 0, max: 1, rest: 1 },
  x: { min: -1, max: 1, rest: 0 },
  y: { min: -1, max: 1, rest: 0 },
  scale: { min: 0, max: 3, rest: 1 },
  volume: { min: 0, max: MAX_VOLUME, rest: 1 },
  pan: { min: -1, max: 1, rest: 0 },
};

const PAD = 5;

const span = (prop: EnvelopeKey) => ENVELOPE_INFO[prop].max - ENVELOPE_INFO[prop].min;

export const envelopeY = (prop: EnvelopeKey, value: number, height: number): number =>
  PAD + (1 - (value - ENVELOPE_INFO[prop].min) / span(prop)) * (height - PAD * 2);

export const envelopeShift = (prop: EnvelopeKey, dy: number, height: number): number =>
  (dy / Math.max(1, height - PAD * 2)) * span(prop);

export type EnvelopeTarget = { nodeId: string; prop: EnvelopeKey; keys: Keyframe[] };

const parseEnvelope = parseOncePer(envelopeSchema);

function readEnvelope(node: WorkflowNode): Envelope | null {
  const raw = node.params.envelope;
  if (raw === undefined) return {};
  if (typeof raw !== "object" || raw === null) return null;
  return parseEnvelope(raw);
}

export function envelopeTarget(
  wf: Workflow,
  nodeId: string | undefined,
  prop: EnvelopeKey | null,
): EnvelopeTarget | null {
  if (!nodeId || !prop) return null;
  const node = findById(wf.nodes, nodeId);
  if (!node || !getNodeDef(node.type)?.params.shape.envelope) return null;
  const envelope = readEnvelope(node);
  return envelope ? { nodeId, prop, keys: envelope[prop] ?? [] } : null;
}

const UNREADABLE_KEYS = "This node's keyframes can't be read. Check them in the Inspector.";

export function setEnvelopeKeys(nodeId: string, prop: EnvelopeKey, keys: Keyframe[]): void {
  const node = findById(useWorkflowStore.getState().workflow?.nodes, nodeId);
  const current = node ? readEnvelope(node) : null;
  if (!current) {
    toast.error(UNREADABLE_KEYS);
    return;
  }
  const envelope = { ...current };
  if (keys.length) envelope[prop] = sortKeys(keys);
  else delete envelope[prop];
  useWorkflowStore.getState().update((wf) => patchNodeParams(wf, nodeId, { envelope }));
}

export function addKeyframe(nodeId: string, prop: EnvelopeKey, keys: Keyframe[], at: number) {
  const frame = wholeFrames(at);
  setEnvelopeKeys(nodeId, prop, [
    ...keys,
    { frame, value: envelopeAt(keys, frame, ENVELOPE_INFO[prop].rest), ease: "linear" },
  ]);
}
