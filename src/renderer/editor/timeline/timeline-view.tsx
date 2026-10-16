import { useEffect, useImperativeHandle, useLayoutEffect, useRef, useState } from "react";
import type { Workflow } from "@/shared/schema/workflow";
import type { Removal } from "@/shared/timeline/edits";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { useFilm } from "../film";
import { AudioBarView, useDuckGains } from "./audio-bar";
import type { BarContext } from "./bar-context";
import { MarkerRow } from "./marker-row";
import { RemoveDialog } from "./remove-dialog";
import { Ruler } from "./ruler";
import { preloadScrub } from "./scrub";
import {
  addMarkerAtPlayhead,
  removeNow,
  removeSelectedMarker,
  selectionRemoval,
  selectNodes,
  splitAtPlayhead,
} from "./timeline-actions";
import { editContext, gestureSalt } from "./timeline-drag";
import {
  AUDIO_H,
  HEADER_W,
  TAIL_PX,
  VIDEO_H,
  VIDEO_TOP,
  audioRowTop,
  inView,
  videoRowTop,
  type Viewport,
} from "./timeline-layout";
import { TimelineRow } from "./timeline-row";
import { useTimeline } from "./timeline-state";
import { TransitionMarks } from "./transition-marks";
import { useBeatFrames } from "./use-beat-frames";
import { useTimelineDrag } from "./use-timeline-drag";
import { useTimelineZoom } from "./use-timeline-zoom";
import { VideoBarView } from "./video-bar";

export type TimelineActions = {
  split: () => void;
  remove: () => void;
  addMarker: () => void;
  zoomBy: (factor: number) => void;
  fit: () => void;
};

const FOLLOW_MARGIN_PX = 24;

function useScrollerView(scroller: React.RefObject<HTMLDivElement | null>) {
  const [view, setView] = useState({ left: 0, width: 800, height: 300 });
  useLayoutEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const update = () =>
      setView({ left: el.scrollLeft, width: el.clientWidth, height: el.clientHeight });
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, [scroller]);
  const onScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const left = e.currentTarget.scrollLeft;
    setView((v) => (v.left === left ? v : { ...v, left }));
  };
  return { view, onScroll };
}

export function TimelineView({
  ref,
  workflow,
  flat,
}: {
  ref: React.Ref<TimelineActions>;
  workflow: Workflow;
  flat: FlatTimeline;
}) {
  const film = useFilm();
  const sourceNodeId = film.sourceNodeId;
  const selectedIds = useUiStore((s) => s.selectedIds);
  const frame = usePlayback((s) => s.frame);
  const tool = useTimeline((s) => s.tool);
  const scroller = useRef<HTMLDivElement>(null);
  const { view, onScroll } = useScrollerView(scroller);
  const [confirm, setConfirm] = useState<Removal | null>(null);

  const { fps, durationInFrames: duration } = flat;
  const selected = new Set(selectedIds);
  const videoLanes = Math.max(1, flat.lanes);
  const audioLanes = Math.max(1, flat.audioLanes);
  const audioTop = VIDEO_TOP + videoLanes * VIDEO_H;
  const layout = { videoLanes, audioTop };
  const totalH = audioTop + audioLanes * AUDIO_H;
  const masterSrc = film.result?.master?.src ?? null;
  const markers = workflow.markers;
  const beats = useBeatFrames(flat);
  const gainFor = useDuckGains(film.spec, film.mix);
  const zoom = useTimelineZoom(scroller, view.width, duration);
  const ppf = zoom.ppf;
  const contentW = Math.max(view.width - HEADER_W, duration * ppf + TAIL_PX);
  const viewport: Viewport = { left: view.left, width: view.width - HEADER_W, pxPerFrame: ppf };
  const drag = useTimelineDrag({
    scroller,
    workflow,
    flat,
    sourceNodeId,
    masterSrc,
    markers,
    beats,
    layout,
    zoom,
  });

  const playing = usePlayback((s) => s.playing);
  useEffect(() => {
    const el = scroller.current;
    if (!el || !playing) return;
    const x = frame * ppf;
    const visibleW = el.clientWidth - HEADER_W;
    if (x < el.scrollLeft || x > el.scrollLeft + visibleW - FOLLOW_MARGIN_PX)
      el.scrollLeft = Math.max(0, x - FOLLOW_MARGIN_PX);
  }, [frame, playing, ppf]);

  useEffect(() => {
    preloadScrub(masterSrc);
  }, [masterSrc]);

  const select = (ids: string[], additive: boolean) => selectNodes(ids, additive, sourceNodeId);
  const newEditContext = (salt: string) => editContext(workflow, fps, sourceNodeId, salt);

  const remove = () => {
    if (removeSelectedMarker()) return;
    const removal = selectionRemoval(newEditContext("d"), flat, selected);
    if (!removal) return;
    if (removal.nodeIds.length > 1) setConfirm(removal);
    else removeNow(removal, sourceNodeId);
  };

  useImperativeHandle(ref, () => ({
    split: () => splitAtPlayhead(newEditContext(gestureSalt()), flat, selected),
    remove,
    addMarker: () => addMarkerAtPlayhead(fps),
    zoomBy: (f) => zoom.zoomAround(f),
    fit: () => useTimeline.getState().fitToView(),
  }));

  const onWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      zoom.zoomAround(e.deltaY < 0 ? 1.15 : 1 / 1.15, drag.xToFrame(e.clientX));
    } else if (!e.shiftKey && Math.abs(e.deltaY) > Math.abs(e.deltaX) && totalH <= view.height) {
      e.currentTarget.scrollLeft += e.deltaY;
    }
  };

  const ctx: BarContext = {
    workflow,
    flat,
    viewport,
    tool,
    selected,
    drag,
    select,
    splitContext: () => newEditContext(gestureSalt()),
  };
  const marquee = drag.marquee;

  return (
    <div
      ref={scroller}
      className="relative h-full overflow-auto select-none"
      onPointerMove={drag.onPointerMove}
      onPointerUp={drag.onPointerUp}
      onPointerCancel={drag.onPointerUp}
      onPointerDown={drag.onEmptyDown}
      onWheel={onWheel}
      onScroll={onScroll}
    >
      <div className="relative" style={{ width: HEADER_W + contentW, height: totalH }}>
        <Ruler
          left={view.left}
          width={view.width}
          contentW={contentW}
          ppf={ppf}
          fps={fps}
          duration={duration}
          onPointerDown={drag.onRulerDown}
        />
        <MarkerRow
          flat={flat}
          markers={markers}
          beats={beats}
          viewport={viewport}
          contentW={contentW}
          onDrag={drag.startMarker}
        />
        {Array.from({ length: videoLanes }, (_, lane) => (
          <TimelineRow
            key={`v${lane}`}
            top={videoRowTop(layout, lane)}
            height={VIDEO_H}
            label={`Video ${lane + 1}`}
            width={contentW}
          >
            {flat.bars
              .filter((b) => b.lane === lane && inView(viewport, b.from, b.to))
              .map((bar) => (
                <VideoBarView key={bar.key} bar={bar} ctx={ctx} />
              ))}
          </TimelineRow>
        ))}
        <TransitionMarks
          workflow={workflow}
          flat={flat}
          viewport={viewport}
          layout={layout}
          onSelect={(id) => select([id], false)}
        />
        {Array.from({ length: audioLanes }, (_, lane) => (
          <TimelineRow
            key={`a${lane}`}
            top={audioRowTop(layout, lane)}
            height={AUDIO_H}
            label={`Sound ${lane + 1}`}
            width={contentW}
          >
            {flat.audio
              .filter((a) => a.lane === lane && inView(viewport, a.from, a.to))
              .map((a) => (
                <AudioBarView key={a.key} bar={a} ctx={ctx} gain={gainFor(a.id)} />
              ))}
          </TimelineRow>
        ))}
        {drag.guide !== null ? (
          <div
            className="pointer-events-none absolute top-0 z-30 w-px bg-keyframe"
            style={{ left: HEADER_W + drag.guide * ppf, height: totalH }}
          />
        ) : null}
        {marquee ? (
          <div
            className="border-primary bg-primary/10 pointer-events-none absolute z-30 border"
            style={{
              left: Math.min(marquee.x0, marquee.x1),
              top: Math.min(marquee.y0, marquee.y1),
              width: Math.abs(marquee.x1 - marquee.x0),
              height: Math.abs(marquee.y1 - marquee.y0),
            }}
          />
        ) : null}
        <div
          className="bg-playhead pointer-events-none absolute top-0 z-30 w-px"
          style={{ left: HEADER_W + frame * ppf, height: Math.max(totalH, view.height) }}
        >
          <div className="bg-playhead absolute -left-1.5 top-0 h-2.5 w-3 [clip-path:polygon(0_0,100%_0,50%_100%)]" />
        </div>
      </div>
      <RemoveDialog
        removal={confirm}
        onConfirm={(removal) => removeNow(removal, sourceNodeId)}
        onClose={() => setConfirm(null)}
      />
    </div>
  );
}
