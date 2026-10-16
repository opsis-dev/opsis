import {
  ArrowLeftRight,
  ArrowRightFromLine,
  Bookmark,
  Gauge,
  GalleryHorizontal,
  Magnet,
  MousePointer2,
  Scissors,
  SquareChartGantt,
  X,
  ZoomIn,
  ZoomOut,
  type LucideIcon,
} from "lucide-react";
import { useRef } from "react";
import { getNodeDef } from "@/shared/nodes/registry";
import { isOutputType } from "@/shared/nodes/output";
import { formatTime } from "@/shared/timeline/snap";
import type { AudioEnvelopeKey, VisualEnvelopeKey } from "@/shared/schema/timeline";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { Kbd } from "@/renderer/components/ui/kbd";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { Separator } from "@/renderer/components/ui/separator";
import { Toggle } from "@/renderer/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { useWorkflowStore } from "../../store/workflow";
import { useFilm } from "../film";
import { useTimeline, type TimelineTool } from "./timeline-state";
import { TimelineView, type TimelineActions } from "./timeline-view";

const TOOLS: Array<{
  id: TimelineTool;
  icon: LucideIcon;
  label: string;
  key: string;
  help: string;
}> = [
  {
    id: "select",
    icon: MousePointer2,
    label: "Select",
    key: "V",
    help: "Move clips and drag their edges to trim. Hold Shift to move later clips too.",
  },
  {
    id: "ripple",
    icon: ArrowRightFromLine,
    label: "Ripple",
    key: "B",
    help: "Moves and trims push the clips after them.",
  },
  {
    id: "roll",
    icon: ArrowLeftRight,
    label: "Roll",
    key: "N",
    help: "Drag the cut between two clips. The total length stays the same.",
  },
  {
    id: "slip",
    icon: GalleryHorizontal,
    label: "Slip",
    key: "Y",
    help: "Shows a different part of a clip without changing where it sits.",
  },
  { id: "blade", icon: Scissors, label: "Split", key: "C", help: "Click a clip to cut it in two." },
  {
    id: "rate",
    icon: Gauge,
    label: "Speed",
    key: "R",
    help: "Drag a clip's edge to speed it up or slow it down.",
  },
];

const VIDEO_KEYS: Array<{ value: VisualEnvelopeKey | "off"; label: string }> = [
  { value: "off", label: "No keyframes" },
  { value: "opacity", label: "Opacity" },
  { value: "x", label: "Move across" },
  { value: "y", label: "Move up and down" },
  { value: "scale", label: "Size" },
];

const AUDIO_KEYS: Array<{ value: AudioEnvelopeKey; label: string }> = [
  { value: "volume", label: "Volume" },
  { value: "pan", label: "Balance" },
];

const TOOL_KEYS: Record<string, TimelineTool> = {
  v: "select",
  b: "ripple",
  n: "roll",
  y: "slip",
  c: "blade",
  r: "rate",
};

const mod = window.demoflow?.platform === "darwin" ? "⌘" : "Ctrl+";

function editingText(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  return !!el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName));
}

export function TimelinePanel() {
  const film = useFilm();
  const workflow = useWorkflowStore((s) => s.workflow);
  const tool = useTimeline((s) => s.tool);
  const snapping = useTimeline((s) => s.snapping);
  const rulerMode = useTimeline((s) => s.rulerMode);
  const videoEnvelope = useTimeline((s) => s.videoEnvelope);
  const audioEnvelope = useTimeline((s) => s.audioEnvelope);
  const frame = usePlayback((s) => s.frame);
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);
  const actions = useRef<TimelineActions | null>(null);

  if (!film.flat || !workflow) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <SquareChartGantt />
          </EmptyMedia>
          <EmptyTitle>No film to show yet</EmptyTitle>
          <EmptyDescription>
            Connect clips to an export, or select a node that makes a clip.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  const fps = film.flat.fps;
  const duration = film.flat.durationInFrames;
  const source = workflow.nodes.find((n) => n.id === film.sourceNodeId);
  const output = workflow.nodes.find((n) => isOutputType(n.type) && !n.disabled);
  const showingExport = !!source && isOutputType(source.type);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (editingText(e.target)) return;
    const pb = usePlayback.getState();
    const ctrl = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    let handled = true;
    if (ctrl && (key === "=" || key === "+")) actions.current?.zoomBy(1.5);
    else if (ctrl && key === "-") actions.current?.zoomBy(1 / 1.5);
    else if (ctrl || (e.altKey && key !== "x")) handled = false;
    else if (key === " ") pb.toggle();
    else if (key === "j") pb.shuttle(-1);
    else if (key === "k") pb.shuttle(0);
    else if (key === "l") pb.shuttle(1);
    else if (key === "arrowleft") pb.step(e.shiftKey ? -fps : -1);
    else if (key === "arrowright") pb.step(e.shiftKey ? fps : 1);
    else if (key === "home") pb.seek(0);
    else if (key === "end") pb.seek(duration - 1);
    else if (key === "i") pb.setIn(pb.frame);
    else if (key === "o") pb.setOut(pb.frame);
    else if (e.altKey && key === "x") {
      pb.setIn(null);
      pb.setOut(null);
    } else if (key === "=" || key === "+") actions.current?.zoomBy(1.5);
    else if (key === "-") actions.current?.zoomBy(1 / 1.5);
    else if (key === "z") actions.current?.fit();
    else if (key === "s") actions.current?.split();
    else if (key === "m") actions.current?.addMarker();
    else if (key === "delete" || key === "backspace") actions.current?.remove();
    else if (TOOL_KEYS[key]) useTimeline.getState().setTool(TOOL_KEYS[key]);
    else handled = false;
    if (handled) {
      e.preventDefault();
      e.stopPropagation();
    }
  };

  return (
    <div
      data-timeline-root
      tabIndex={0}
      className="flex h-full flex-col outline-none"
      onKeyDown={onKeyDown}
    >
      <div className="flex h-9 shrink-0 items-center gap-1 overflow-x-auto border-b px-2">
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[tool]}
          onValueChange={(v) => v[0] && useTimeline.getState().setTool(v[0] as TimelineTool)}
        >
          {TOOLS.map((t) => (
            <Tooltip key={t.id}>
              <TooltipTrigger render={<ToggleGroupItem value={t.id} aria-label={t.label} />}>
                <t.icon />
              </TooltipTrigger>
              <TooltipContent className="max-w-64">
                {t.label} <Kbd>{t.key}</Kbd>
                <div className="text-muted-foreground mt-1">{t.help}</div>
              </TooltipContent>
            </Tooltip>
          ))}
        </ToggleGroup>
        <Tooltip>
          <TooltipTrigger
            render={
              <Toggle
                size="sm"
                aria-label="Snap"
                pressed={snapping}
                onPressedChange={(v) => useTimeline.getState().setSnapping(v)}
              />
            }
          >
            <Magnet />
          </TooltipTrigger>
          <TooltipContent>
            Snap to the playhead, clip edges, markers, actions and beats. Hold <Kbd>Alt</Kbd> to
            turn it off while dragging.
          </TooltipContent>
        </Tooltip>
        <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Split at the playhead"
                onClick={() => actions.current?.split()}
              />
            }
          >
            <Scissors />
          </TooltipTrigger>
          <TooltipContent>
            Split at the playhead <Kbd>S</Kbd>
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Add a marker"
                onClick={() => actions.current?.addMarker()}
              />
            }
          >
            <Bookmark />
          </TooltipTrigger>
          <TooltipContent>
            Add a marker <Kbd>M</Kbd>
          </TooltipContent>
        </Tooltip>
        <Select
          value={videoEnvelope ?? "off"}
          onValueChange={(v) =>
            useTimeline.getState().setVideoEnvelope(v === "off" ? null : (v as VisualEnvelopeKey))
          }
        >
          <SelectTrigger size="sm" className="w-36" aria-label="Clip keyframes">
            <SelectValue>
              {(v: string) => VIDEO_KEYS.find((k) => k.value === v)?.label ?? "No keyframes"}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {VIDEO_KEYS.map((k) => (
                <SelectItem key={k.value} value={k.value}>
                  {k.label}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <Select
          value={audioEnvelope}
          onValueChange={(v) => useTimeline.getState().setAudioEnvelope(v as AudioEnvelopeKey)}
        >
          <SelectTrigger size="sm" className="w-28" aria-label="Sound keyframes">
            <SelectValue>{(v: string) => AUDIO_KEYS.find((k) => k.value === v)?.label}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {AUDIO_KEYS.map((k) => (
                <SelectItem key={k.value} value={k.value}>
                  {k.label}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
        <div className="flex-1" />
        {source ? (
          <Badge variant="secondary" className="max-w-40 truncate">
            {source.label ?? getNodeDef(source.type)?.title}
          </Badge>
        ) : null}
        {!showingExport && output ? (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => useUiStore.getState().pinSource(output.id)}
          >
            Show the export
          </Button>
        ) : null}
        {inPoint !== null || outPoint !== null ? (
          <Badge variant="outline" className="gap-1 tabular-nums">
            {formatTime(inPoint ?? 0, fps, rulerMode)}–
            {formatTime(outPoint ?? duration, fps, rulerMode)}
            <button
              type="button"
              aria-label="Clear the loop"
              onClick={() => {
                usePlayback.getState().setIn(null);
                usePlayback.getState().setOut(null);
              }}
            >
              <X className="size-3" />
            </button>
          </Badge>
        ) : null}
        <Tooltip>
          <TooltipTrigger
            render={
              <button
                type="button"
                className="text-muted-foreground hover:text-foreground px-1 text-xs whitespace-nowrap tabular-nums"
                onClick={() =>
                  useTimeline
                    .getState()
                    .setRulerMode(rulerMode === "seconds" ? "timecode" : "seconds")
                }
              />
            }
          >
            {formatTime(frame, fps, rulerMode)} / {formatTime(duration, fps, rulerMode)}
          </TooltipTrigger>
          <TooltipContent>Switch between seconds and timecode</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Zoom out"
                onClick={() => actions.current?.zoomBy(1 / 1.5)}
              />
            }
          >
            <ZoomOut />
          </TooltipTrigger>
          <TooltipContent>
            Zoom out <Kbd>{mod}−</Kbd>
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="sm"
                className="px-2"
                onClick={() => actions.current?.fit()}
              />
            }
          >
            Fit
          </TooltipTrigger>
          <TooltipContent>
            Show the whole film <Kbd>Z</Kbd>
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Zoom in"
                onClick={() => actions.current?.zoomBy(1.5)}
              />
            }
          >
            <ZoomIn />
          </TooltipTrigger>
          <TooltipContent>
            Zoom in <Kbd>{mod}=</Kbd>
          </TooltipContent>
        </Tooltip>
      </div>
      <div className="min-h-0 flex-1">
        <TimelineView ref={actions} />
      </div>
    </div>
  );
}
