import { samplePoints } from "@opsis/video/common/envelope";
import { uniqueKeys } from "@opsis/video/common/lists";
import { round3 } from "@opsis/video/common/math";
import type { Keyframe } from "@/generated/bindings";
import type { EnvelopeKey } from "@/generated/bindings";
import { envelopeTrack } from "@/timeline/envelopes";
import type { BarContext } from "./bar-context";
import { editKeys, envelopeTarget, envelopeY, type EnvelopeTarget } from "./envelope";
import type { KeyStart } from "./timeline-drag";
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
    editKeys({
      op: "addKey",
      nodeId: target.nodeId,
      track: target.prop,
      frame: localAt(e.clientX),
    });
  };
}

type EnvelopeMapping = {
  toX: (local: number) => number;
  toLocal: (abs: number) => number;
  scale?: number;
};

export type BarEnvelope = EnvelopeMapping & {
  target: EnvelopeTarget | null;
  addKey: (e: React.PointerEvent) => void;
};

export function useBarEnvelope(
  ctx: BarContext,
  nodeId: string | undefined,
  prop: EnvelopeKey | null,
  mapping: EnvelopeMapping,
): BarEnvelope {
  const target = envelopeTarget(ctx.workflow, nodeId, prop);
  const addKey = useKeyframeAdder(target, (clientX) => mapping.toLocal(ctx.drag.xToFrame(clientX)));
  return { ...mapping, target, addKey };
}

export function EnvelopeOverlay({
  envelope,
  width,
  height,
  startKey,
}: {
  envelope: BarEnvelope;
  width: number;
  height: number;
  startKey: (e: React.PointerEvent, key: KeyStart) => void;
}) {
  const isDouble = useDoublePress();
  const { target, toX, toLocal, scale = 1 } = envelope;
  if (!target) return null;
  const { nodeId, prop, keys } = target;
  const { rest, label } = envelopeTrack(prop);
  const y = (v: number) => envelopeY(prop, v * scale, height);
  const ids = uniqueKeys(keys.map((k) => String(k.frame)));
  const remove = (index: number) => editKeys({ op: "removeKey", nodeId, track: prop, index });
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
            startKey(e, { nodeId, prop, keys, index, toLocal, height, scale });
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
