import { useEffect, useLayoutEffect, useState } from "react";
import { cn } from "@/renderer/lib/utils";
import type { Workflow } from "@/shared/schema/workflow";
import type { FlatTimeline } from "@/shared/timeline/flatten";
import { usePlayback } from "../../store/playback";
import { selectedSet, useUiStore } from "../../store/ui";
import type { Film } from "../film";
import { AudioBarView } from "./audio-bar";
import type { BarContext } from "./bar-context";
import { MarkerRow } from "./marker-row";
import { Ruler } from "./ruler";
import { preloadScrub } from "./scrub";
import { selectNodes } from "./timeline-actions";
import type { MarqueeBox } from "./timeline-drag";
import {
  AUDIO_H,
  HEADER_W,
  VIDEO_H,
  audioRowTop,
  inView,
  timelineGeometry,
  videoRowTop,
  type Viewport,
} from "./timeline-layout";
import { TimelineRow } from "./timeline-row";
import { useTimeline } from "./timeline-state";
import { TransitionMarks } from "./transition-marks";
import { useBeatFrames } from "./use-beat-frames";
import { useDuckGains } from "./use-duck-gains";
import type { TimelineCommands } from "./use-timeline-commands";
import { useTimelineDrag } from "./use-timeline-drag";
import type { TimelineZoom } from "./use-timeline-zoom";
import { VideoBarView } from "./video-bar";

export type TimelineFilm = Film & { flat: FlatTimeline };

const FOLLOW_MARGIN_PX = 24;
const INITIAL_VIEW = { left: 0, width: 800, height: 300 };

type ScrollerView = ReturnType<typeof useScrollerView>;

export function useScrollerView(scroller: React.RefObject<HTMLDivElement | null>) {
  const [view, setView] = useState(INITIAL_VIEW);
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

function useFollowPlayhead(scroller: React.RefObject<HTMLDivElement | null>, ppf: number) {
  useEffect(
    () =>
      usePlayback.subscribe(({ frame, playing }) => {
        const el = scroller.current;
        if (!el || !playing) return;
        const x = frame * ppf;
        const visibleW = el.clientWidth - HEADER_W;
        if (x < el.scrollLeft || x > el.scrollLeft + visibleW - FOLLOW_MARGIN_PX)
          el.scrollLeft = Math.max(0, x - FOLLOW_MARGIN_PX);
      }),
    [scroller, ppf],
  );
}

function FrameLine({
  frame,
  ppf,
  height,
  className,
  children,
}: {
  frame: number;
  ppf: number;
  height: number;
  className: string;
  children?: React.ReactNode;
}) {
  return (
    <div
      className={cn("pointer-events-none absolute top-0 z-30 w-px", className)}
      style={{ left: HEADER_W + frame * ppf, height }}
    >
      {children}
    </div>
  );
}

function Playhead({ ppf, height }: { ppf: number; height: number }) {
  const frame = usePlayback((s) => s.frame);
  return (
    <FrameLine frame={frame} ppf={ppf} height={height} className="bg-playhead">
      <div className="bg-playhead absolute -left-1.5 top-0 h-2.5 w-3 [clip-path:polygon(0_0,100%_0,50%_100%)]" />
    </FrameLine>
  );
}

function Marquee({ box }: { box: MarqueeBox }) {
  return (
    <div
      className="border-primary bg-primary/10 pointer-events-none absolute z-30 border"
      style={{
        left: Math.min(box.x0, box.x1),
        top: Math.min(box.y0, box.y1),
        width: Math.abs(box.x1 - box.x0),
        height: Math.abs(box.y1 - box.y0),
      }}
    />
  );
}

type Laned = { key: string; lane: number; from: number; to: number };

function BarLanes<T extends Laned>({
  count,
  name,
  top,
  height,
  bars,
  contentW,
  viewport,
  render,
}: {
  count: number;
  name: string;
  top: (lane: number) => number;
  height: number;
  bars: T[];
  contentW: number;
  viewport: Viewport;
  render: (bar: T) => React.ReactNode;
}) {
  return Array.from({ length: count }, (_, lane) => (
    <TimelineRow
      key={lane}
      top={top(lane)}
      height={height}
      label={`${name} ${lane + 1}`}
      width={contentW}
    >
      {bars.filter((b) => b.lane === lane && inView(viewport, b.from, b.to)).map(render)}
    </TimelineRow>
  ));
}

type TimelineViewProps = {
  scroller: React.RefObject<HTMLDivElement | null>;
  scroll: ScrollerView;
  zoom: TimelineZoom;
  film: TimelineFilm;
  workflow: Workflow;
  commands: TimelineCommands;
};

function useTimelineScene({ scroller, scroll, zoom, film, workflow, commands }: TimelineViewProps) {
  const { flat, sourceNodeId } = film;
  const selected = useUiStore((s) => selectedSet(s.selectedIds));
  const tool = useTimeline((s) => s.tool);
  const geometry = timelineGeometry(flat, scroll.view, zoom.ppf);
  const masterSrc = film.result?.master?.src ?? null;
  const beats = useBeatFrames(flat);
  const gainFor = useDuckGains(film);
  const drag = useTimelineDrag({
    scroller,
    workflow,
    flat,
    sourceNodeId,
    masterSrc,
    markers: workflow.markers,
    beats,
    layout: geometry.layout,
    zoom,
  });

  useFollowPlayhead(scroller, zoom.ppf);

  useEffect(() => {
    preloadScrub(masterSrc);
  }, [masterSrc]);

  const select = (ids: string[], additive: boolean) => selectNodes(ids, additive, sourceNodeId);
  const ctx: BarContext = {
    workflow,
    flat,
    viewport: geometry.viewport,
    tool,
    selected,
    drag,
    select,
    splitAt: commands.splitAt,
  };
  return { ...geometry, beats, gainFor, drag, select, ctx };
}

export function TimelineView(props: TimelineViewProps) {
  const { scroller, scroll, zoom, film, workflow } = props;
  const { flat } = film;
  const { view, onScroll } = scroll;
  const { fps, durationInFrames: duration } = flat;
  const ppf = zoom.ppf;
  const { layout, audioLanes, totalH, contentW, viewport, beats, gainFor, drag, select, ctx } =
    useTimelineScene(props);
  const markers = workflow.markers;

  return (
    <div
      ref={scroller}
      className="relative h-full overflow-auto select-none"
      {...drag.scrollerHandlers}
      onPointerDown={drag.onEmptyDown}
      onWheel={zoom.onWheel}
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
        <BarLanes
          count={layout.videoLanes}
          name="Video"
          top={(lane) => videoRowTop(layout, lane)}
          height={VIDEO_H}
          bars={flat.bars}
          contentW={contentW}
          viewport={viewport}
          render={(bar) => <VideoBarView key={bar.key} bar={bar} ctx={ctx} />}
        />
        <TransitionMarks
          workflow={workflow}
          flat={flat}
          viewport={viewport}
          layout={layout}
          onSelect={(id) => select([id], false)}
        />
        <BarLanes
          count={audioLanes}
          name="Sound"
          top={(lane) => audioRowTop(layout, lane)}
          height={AUDIO_H}
          bars={flat.audio}
          contentW={contentW}
          viewport={viewport}
          render={(a) => <AudioBarView key={a.key} bar={a} ctx={ctx} gain={gainFor(a.id)} />}
        />
        {drag.guide !== null ? (
          <FrameLine frame={drag.guide} ppf={ppf} height={totalH} className="bg-keyframe" />
        ) : null}
        {drag.marquee ? <Marquee box={drag.marquee} /> : null}
        <Playhead ppf={ppf} height={Math.max(totalH, view.height)} />
      </div>
    </div>
  );
}
