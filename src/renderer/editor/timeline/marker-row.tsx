import { useState } from "react";
import type { Marker } from "@/shared/schema/workflow";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import { Input } from "@/renderer/components/ui/input";
import { cn } from "@/renderer/lib/utils";
import { renameMarker } from "./timeline-actions";
import { MARKERS_H, RULER_H, inView, type Viewport } from "./timeline-layout";
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
  const [renaming, setRenaming] = useState(false);
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
        if (isDouble(e)) setRenaming(true);
        else onDrag(e, marker);
      }}
    >
      {renaming ? (
        <Input
          autoFocus
          defaultValue={marker.label}
          className="h-4 w-28 px-1 text-[10px]"
          onPointerDown={(e) => e.stopPropagation()}
          onKeyDown={(e) => {
            e.stopPropagation();
            if (e.key === "Enter" || e.key === "Escape") e.currentTarget.blur();
          }}
          onBlur={(e) => {
            const label = e.currentTarget.value.trim() || marker.label;
            setRenaming(false);
            if (label !== marker.label) renameMarker(marker.id, label);
          }}
        />
      ) : (
        marker.label
      )}
    </div>
  );
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
      {beats
        .filter((f) => inView(viewport, f, f))
        .map((f, i) => (
          <div
            key={`b${i}`}
            className="bg-track-music absolute bottom-0 h-1.5 w-px opacity-80"
            style={{ left: f * ppf }}
          />
        ))}
      {flat.marks
        .filter((m) => m.kind === "chapter" && inView(viewport, m.frame, m.frame))
        .map((m) => (
          <div
            key={m.key}
            title={m.label}
            className="text-muted-foreground absolute top-0.5 flex h-4 items-center gap-1 text-[10px] whitespace-nowrap"
            style={{ left: m.frame * ppf }}
          >
            <span className="bg-muted-foreground/70 h-3 w-0.5" />
            {m.label}
          </div>
        ))}
      {markers.map((m) => (
        <MarkerTag key={m.id} marker={m} x={(m.atMs / 1000) * flat.fps * ppf} onDrag={onDrag} />
      ))}
    </TimelineRow>
  );
}
