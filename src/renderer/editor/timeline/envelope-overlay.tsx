import { round3 } from "@/shared/math";
import type { Keyframe } from "@/shared/schema/timeline";
import type { Workflow } from "@/shared/schema/workflow";
import { samplePoints } from "@/shared/timeline/envelope";
import {
  addKeyframe,
  ENVELOPE_INFO,
  envelopeTarget,
  envelopeY,
  setEnvelopeKeys,
  type EnvelopeKey,
  type EnvelopeTarget,
} from "./envelope";
import type { KeyStart } from "./use-timeline-drag";
import { useDoublePress } from "./use-double-press";

type LineShape = {
  keys: Keyframe[];
  width: number;
  rest: number;
  toX: (local: number) => number;
  y: (value: number) => number;
};

function envelopeLine({ keys, width, rest, toX, y }: LineShape): string {
  const samples = samplePoints(keys);
  const first = samples.at(0)?.value ?? rest;
  const last = samples.at(-1)?.value ?? rest;
  return [
    `0,${y(first)}`,
    ...samples.map((p) => `${toX(p.frame)},${y(p.value)}`),
    `${width},${y(last)}`,
  ].join(" ");
}

function keyIds(keys: Keyframe[]): string[] {
  const seen = new Map<number, number>();
  return keys.map((k) => {
    const n = seen.get(k.frame) ?? 0;
    seen.set(k.frame, n + 1);
    return n ? `${k.frame}#${n}` : String(k.frame);
  });
}

const onKeyframe = (e: React.PointerEvent) =>
  e.target instanceof Element && e.target.closest("[data-keyframe]") !== null;

function useKeyframeAdder(
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

export function useBarEnvelope(
  workflow: Workflow,
  nodeId: string | undefined,
  prop: EnvelopeKey | null,
  localAt: (clientX: number) => number,
) {
  const target = envelopeTarget(workflow, nodeId, prop);
  const addKey = useKeyframeAdder(target, localAt);
  return { target, addKey };
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
  const { label, rest } = ENVELOPE_INFO[prop];
  const isDouble = useDoublePress();
  const y = (v: number) => envelopeY(prop, v, height);
  const ids = keyIds(keys);
  const remove = (index: number) =>
    setEnvelopeKeys(
      nodeId,
      prop,
      keys.filter((_key, k) => k !== index),
    );
  return (
    <svg className="absolute inset-0 overflow-visible" width={width} height={height}>
      <polyline
        points={envelopeLine({ keys, width, rest, toX, y })}
        fill="none"
        className="stroke-keyframe pointer-events-none"
        strokeWidth={1.25}
      />
      {keys.map((k, index) => (
        <circle
          key={ids[index]}
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
