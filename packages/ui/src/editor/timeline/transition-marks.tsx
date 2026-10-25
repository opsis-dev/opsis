import { percentText } from "@opsis/core/format";
import { coversTooMuch } from "@opsis/core/nodes/card";
import { nodeNameById } from "@opsis/core/nodes/registry";
import { transitionTitle } from "@opsis/core/nodes/transitions";
import type { Workflow } from "@opsis/core/schema/workflow";
import type { FlatTimeline, TransitionMark } from "@opsis/core/timeline/flatten";
import { cn } from "@/lib/utils";
import {
  BAR_INSET,
  HEADER_W,
  VIDEO_H,
  inView,
  videoRowTop,
  type LaneLayout,
  type Viewport,
} from "./timeline-layout";

const MIN_LABEL_PX = 40;
const MIN_TRANSITION_PX = 4;

function transitionTip(wf: Workflow, t: TransitionMark): string {
  const slot = t.slot;
  const edge = slot
    ? wf.edges.find((e) => e.target === slot.nodeId && e.targetHandle === slot.port)
    : undefined;
  const into = edge ? ` into ${nodeNameById(wf.nodes, edge.source, "")}` : "";
  const cover = coversTooMuch(t.covers)
    ? `. It covers ${percentText(t.covers, { max: 100 })} of a clip: turn on Keep every clip whole, or shorten it`
    : "";
  return `${transitionTitle(t.kind)}${into}${cover}.`;
}

export function TransitionMarks({
  workflow,
  flat,
  viewport,
  layout,
  onSelect,
}: {
  workflow: Workflow;
  flat: FlatTimeline;
  viewport: Viewport;
  layout: LaneLayout;
  onSelect: (nodeId: string) => void;
}) {
  const ppf = viewport.pxPerFrame;
  return flat.transitions
    .filter((t) => inView(viewport, t.from, t.to) && t.lanes.length)
    .map((t) => {
      const top = Math.max(...t.lanes);
      const bottom = Math.min(...t.lanes);
      return (
        <div
          key={t.key}
          className={cn(
            "pointer-events-none absolute z-10 flex items-end justify-center overflow-hidden rounded-sm border bg-[repeating-linear-gradient(135deg,var(--transition-hatch)_0,var(--transition-hatch)_4px,transparent_4px,transparent_8px)]",
            coversTooMuch(t.covers) ? "border-warning" : "border-foreground/30",
          )}
          style={{
            left: HEADER_W + t.from * ppf,
            width: Math.max(MIN_TRANSITION_PX, (t.to - t.from) * ppf),
            top: videoRowTop(layout, top) + BAR_INSET / 2,
            height: (top - bottom + 1) * VIDEO_H - BAR_INSET,
          }}
        >
          {(t.to - t.from) * ppf > MIN_LABEL_PX ? (
            <button
              type="button"
              className="bg-background/70 hover:bg-background pointer-events-auto mb-0.5 rounded px-1 text-[10px] whitespace-nowrap"
              title={transitionTip(workflow, t)}
              onPointerDown={(e) => e.stopPropagation()}
              onClick={() => {
                if (t.slot) onSelect(t.slot.nodeId);
              }}
            >
              {transitionTitle(t.kind)}
            </button>
          ) : null}
        </div>
      );
    });
}
