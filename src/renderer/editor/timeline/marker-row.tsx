import { cn } from "@/renderer/lib/utils";
import type { Marker } from "@/shared/schema/workflow";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import { useRenaming } from "../inline-edit";
import { patchMarker } from "./timeline-actions";
import { MARKERS_H, RULER_H, inView, markerFrame, type Viewport } from "./timeline-layout";
import { TimelineRow } from "./timeline-row";
import { useTimeline } from "./timeline-state";
import { useDoublePress } from "./use-double-press";

function MarkerTag({
  marker,
  x,
  onDrag,
}: {
  marker: Marker;
  x: number;
  onDrag: (e: React.PointerEvent, marker: Marker) => void;
}) {
  const selected = useTimeline((s) => s.selectedMarker === marker.id);
  const { input, startRenaming } = useRenaming(
    marker.label,
    (label) => patchMarker(marker.id, { label }),
    "h-4 w-28 px-1 text-[10px]",
  );
  const isDouble = useDoublePress();
  return (
    <div
      className={cn(
        "absolute top-0.5 flex h-4 cursor-grab items-center rounded-r px-1 text-[10px] whitespace-nowrap",
        selected ? "bg-playhead text-background" : "bg-accent text-foreground",
      )}
      style={{ left: x }}
      onPointerDown={(e) => {
        e.stopPropagation();
        if (e.button !== 0) return;
        if (isDouble(e)) startRenaming();
        else onDrag(e, marker);
      }}
    >
      {input ?? marker.label}
    </div>
  );
}

function BeatTicks({ beats, viewport }: { beats: number[]; viewport: Viewport }) {
  return beats
    .filter((f) => inView(viewport, f, f))
    .map((f, i) => (
      <div
        key={`b${i}`}
        className="bg-track-music absolute bottom-0 h-1.5 w-px opacity-80"
        style={{ left: f * viewport.pxPerFrame }}
      />
    ));
}

function ChapterLabels({ marks, viewport }: { marks: FlatTimeline["marks"]; viewport: Viewport }) {
  return marks
    .filter((m) => m.kind === "chapter" && inView(viewport, m.frame, m.frame))
    .map((m) => (
      <div
        key={m.key}
        title={m.label}
        className="text-muted-foreground absolute top-0.5 flex h-4 items-center gap-1 text-[10px] whitespace-nowrap"
        style={{ left: m.frame * viewport.pxPerFrame }}
      >
        <span className="bg-muted-foreground/70 h-3 w-0.5" />
        {m.label}
      </div>
    ));
}

export function MarkerRow({
  flat,
  markers,
  beats,
  viewport,
  contentW,
  onDrag,
}: {
  flat: FlatTimeline;
  markers: Marker[];
  beats: number[];
  viewport: Viewport;
  contentW: number;
  onDrag: (e: React.PointerEvent, marker: Marker) => void;
}) {
  const ppf = viewport.pxPerFrame;
  return (
    <TimelineRow top={RULER_H} height={MARKERS_H} label="Markers" width={contentW}>
      <BeatTicks beats={beats} viewport={viewport} />
      <ChapterLabels marks={flat.marks} viewport={viewport} />
      {markers.map((m) => (
        <MarkerTag key={m.id} marker={m} x={markerFrame(m, flat.fps) * ppf} onDrag={onDrag} />
      ))}
    </TimelineRow>
  );
}
