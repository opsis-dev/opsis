import { isNumber, isOneOf, isRecord } from "@opsis/video/common/guards";
import type { CameraPath, Ease, EnvelopeKey, EnvelopeTrack, Keyframe } from "@/generated/bindings";
import { timelineBook } from "@/store/catalog";

export type Envelope = Partial<Record<EnvelopeKey, Keyframe[]>>;

const isEase = (v: unknown): v is Ease =>
  isOneOf(
    timelineBook().eases.map(([ease]) => ease),
    v,
  );

const isKeyframe = (v: unknown): boolean =>
  isRecord(v) && isNumber(v.frame) && isNumber(v.value) && isEase(v.ease);

const isCameraKey = (v: unknown): boolean =>
  isRecord(v) && [v.frame, v.yaw, v.pitch, v.distance, v.height].every(isNumber) && isEase(v.ease);

export function envelopeTrack(key: EnvelopeKey): EnvelopeTrack {
  const track = timelineBook().envelopes.find((t) => t.key === key);
  if (!track) throw new Error(`The editor catalog has no keyframe track “${key}”.`);
  return track;
}

export const isEnvelopeKey = (key: string): key is EnvelopeKey =>
  timelineBook().envelopes.some((t) => t.key === key);

export const isCameraPath = (value: unknown): value is CameraPath =>
  isRecord(value) && Array.isArray(value.keys) && value.keys.every(isCameraKey);

export const isEnvelope = (value: unknown): value is Envelope =>
  isRecord(value) &&
  Object.entries(value).every(
    ([key, keys]) =>
      isEnvelopeKey(key) && (keys === undefined || (Array.isArray(keys) && keys.every(isKeyframe))),
  );
