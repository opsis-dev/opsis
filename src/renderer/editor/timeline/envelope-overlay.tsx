import { round3 } from "@/shared/math";
import type { Keyframe } from "@/shared/schema/timeline";
import { samplePoints } from "@/shared/timeline/envelope";
import { addKeyframe, ENVELOPE_INFO, setEnvelopeKeys, type EnvelopeKey } from "./envelope";
import type { KeyStart } from "./use-timeline-drag";
import { useDoublePress } from "./use-double-press";

const PAD = 5;

export type EnvelopeTarget = { nodeId: string; prop: EnvelopeKey; keys: Keyframe[] };

function envelopeLine(
  keys: Keyframe[],
  width: number,
  rest: number,
  toX: (local: number) => number,
  y: (value: number) => number,
): string {
  const samples = samplePoints(keys);
  if (!samples.length) return `0,${y(rest)} ${width},${y(rest)}`;
  const first = samples[0].value;
  const last = samples[samples.length - 1].value;
  return [
    `0,${y(first)}`,
    ...samples.map((p) => `${toX(p.frame)},${y(p.value)}`),
    `${width},${y(last)}`,
  ].join(" ");
}

const onKeyframe = (e: React.PointerEvent) =>
  e.target instanceof Element && e.target.closest("[data-keyframe]") !== null;

export function useKeyframeAdder(
  target: EnvelopeTarget | null,
  localAt: (clientX: number) => number,
): (e: React.PointerEvent) => void {
  const isDouble = useDoublePress();
  return (e) => {
    if (!target || e.button !== 0 || onKeyframe(e) || !isDouble(e)) return;
    e.stopPropagation();
    addKeyframe(target.nodeId, target.prop, target.keys, localAt(e.clientX));
  };
}

export function EnvelopeOverlay({
  target,
  width,
  height,
  toX,
  toLocal,
  startKey,
}: {
  target: EnvelopeTarget;
  width: number;
  height: number;
  toX: (local: number) => number;
  toLocal: (abs: number) => number;
  startKey: (e: React.PointerEvent, key: KeyStart) => void;
}) {
  const { nodeId, prop, keys } = target;
  const { label, min, max, rest } = ENVELOPE_INFO[prop];
  const isDouble = useDoublePress();
  const y = (v: number) => PAD + (1 - (v - min) / (max - min)) * (height - PAD * 2);
  const remove = (index: number) =>
    setEnvelopeKeys(
      nodeId,
      prop,
      keys.filter((_key, k) => k !== index),
    );
  return (
    <svg className="absolute inset-0 overflow-visible" width={width} height={height}>
      <polyline
        points={envelopeLine(keys, width, rest, toX, y)}
        fill="none"
        className="stroke-keyframe pointer-events-none"
        strokeWidth={1.25}
      />
      {keys.map((k, index) => (
        <circle
          key={index}
          data-keyframe
          cx={toX(k.frame)}
          cy={y(k.value)}
          r={4}
          className="fill-keyframe stroke-background cursor-move"
          aria-label={`${label} ${round3(k.value)}`}
          onPointerDown={(e) => {
            if (e.button === 0 && isDouble(e)) {
              e.stopPropagation();
              remove(index);
              return;
            }
            startKey(e, { nodeId, prop, keys, index, toLocal, height });
          }}
        >
          <title>
            {`${label} ${round3(k.value)}. Drag to change, double-click to delete. Click to type exact values in the Inspector.`}
          </title>
        </circle>
      ))}
    </svg>
  );
}
