import { isRecord } from "@opsis/video/common/guards";
import { findById } from "@opsis/video/common/lists";
import type {
  EnvelopeKey,
  Keyframe,
  KeyframeEdit,
  Workflow,
  WorkflowNode,
} from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { envelopeTrack, isEnvelope, type Envelope } from "@/timeline/envelopes";
import { commitEdit } from "./timeline-actions";

const PAD = 5;

const span = (prop: EnvelopeKey) => envelopeTrack(prop).max - envelopeTrack(prop).min;

export const envelopeY = (prop: EnvelopeKey, value: number, height: number): number =>
  PAD + (1 - (value - envelopeTrack(prop).min) / span(prop)) * (height - PAD * 2);

export const envelopeShift = (prop: EnvelopeKey, dy: number, height: number): number =>
  (dy / Math.max(1, height - PAD * 2)) * span(prop);

export type EnvelopeTarget = { nodeId: string; prop: EnvelopeKey; keys: Keyframe[] };

const hasEnvelope = (type: string): boolean => {
  const params = getNodeDef(type)?.params;
  return isRecord(params) && isRecord(params.properties) && "envelope" in params.properties;
};

function readEnvelope(node: WorkflowNode): Envelope | null {
  const raw = node.params.envelope;
  if (raw === undefined) return {};
  return isEnvelope(raw) ? raw : null;
}

export function envelopeTarget(
  wf: Workflow,
  nodeId: string | undefined,
  prop: EnvelopeKey | null,
): EnvelopeTarget | null {
  if (!nodeId || !prop) return null;
  const node = findById(wf.nodes, nodeId);
  if (!node || !hasEnvelope(node.type)) return null;
  const envelope = readEnvelope(node);
  return envelope ? { nodeId, prop, keys: envelope[prop] ?? [] } : null;
}

export const editKeys = (edit: KeyframeEdit): void => commitEdit({ op: "keyframes", edit });
