import { isNumber, isOneOf, isRecord } from "@opsis/video/common/guards";
import type { CameraPath, Ease, EnvelopeKey, EnvelopeTrack, Keyframe } from "@/generated/bindings";
import { timelineBook } from "@/store/catalog";

export type Envelope = Partial<Record<EnvelopeKey, Keyframe[]>>;

function isEase(v: unknown): v is Ease {
  return isOneOf(
    timelineBook().eases.map(([ease]) => ease),
    v,
  );
}

function isKeyframe(v: unknown): boolean {
  return isRecord(v) && isNumber(v.frame) && isNumber(v.value) && isEase(v.ease);
}

function isCameraKey(v: unknown): boolean {
  return (
    isRecord(v) && [v.frame, v.yaw, v.pitch, v.distance, v.height].every(isNumber) && isEase(v.ease)
  );
}

export function envelopeTrack(key: EnvelopeKey): EnvelopeTrack {
  const track = timelineBook().envelopes.find((t) => t.key === key);
  if (!track) throw new Error(`The editor catalog has no keyframe track “${key}”.`);
  return track;
}

export function isEnvelopeKey(key: string): key is EnvelopeKey {
  return timelineBook().envelopes.some((t) => t.key === key);
}

export function isCameraPath(value: unknown): value is CameraPath {
  return isRecord(value) && Array.isArray(value.keys) && value.keys.every(isCameraKey);
}

export function isEnvelope(value: unknown): value is Envelope {
  return (
    isRecord(value) &&
    Object.entries(value).every(
      ([key, keys]) =>
        isEnvelopeKey(key) &&
        (keys === undefined || (Array.isArray(keys) && keys.every(isKeyframe))),
    )
  );
}
