import {
  ArrowLeftRight,
  ArrowRightFromLine,
  Bookmark,
  Gauge,
  GalleryHorizontal,
  Magnet,
  MousePointer2,
  Scissors,
  X,
  ZoomIn,
  ZoomOut,
  type LucideIcon,
} from "lucide-react";
import { nodeName } from "@/shared/nodes/registry";
import { isOutputType } from "@/shared/nodes/output";
import {
  AUDIO_ENVELOPES,
  VISUAL_ENVELOPES,
  type VisualEnvelopeKey,
} from "@/shared/schema/timeline";
import type { Workflow } from "@/shared/schema/workflow";
import { formatTime } from "@/shared/timeline/time";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
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
import { shortcutLabel } from "../../app/app-bar";
import { usePlayback } from "../../store/playback";
import { useUiStore } from "../../store/ui";
import { ENVELOPE_INFO } from "./envelope";
import { useTimeline, type TimelineTool } from "./timeline-state";
import type { TimelineActions } from "./timeline-view";

export const ZOOM_STEP = 1.5;

export const TOOLS: Array<{
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

type VideoChoice = VisualEnvelopeKey | "off";

const VIDEO_CHOICES: VideoChoice[] = ["off", ...VISUAL_ENVELOPES];

const videoLabel = (v: VideoChoice) => (v === "off" ? "No keyframes" : ENVELOPE_INFO[v].label);

function IconTip({
  label,
  tip,
  onClick,
  children,
}: {
  label?: string;
  tip: React.ReactNode;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size={label ? "icon-sm" : "sm"}
            className={label ? undefined : "px-2"}
            aria-label={label}
            onClick={onClick}
          />
        }
      >
        {children}
      </TooltipTrigger>
      <TooltipContent>{tip}</TooltipContent>
    </Tooltip>
  );
}

function ToolPicker() {
  const tool = useTimeline((s) => s.tool);
  const snapping = useTimeline((s) => s.snapping);
  return (
    <>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={[tool]}
        onValueChange={(v) => {
          const next = TOOLS.find((t) => t.id === v[0]);
          if (next) useTimeline.getState().setTool(next.id);
        }}
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
          Snap to the playhead, clip edges, markers, actions and beats. Hold <Kbd>Alt</Kbd> to turn
          it off while dragging.
        </TooltipContent>
      </Tooltip>
    </>
  );
}

function EnvelopePickers() {
  const videoChoice: VideoChoice = useTimeline((s) => s.videoEnvelope) ?? "off";
  const audioEnvelope = useTimeline((s) => s.audioEnvelope);
  return (
    <>
      <Select
        value={videoChoice}
        onValueChange={(v) => {
          if (v) useTimeline.getState().setVideoEnvelope(v === "off" ? null : v);
        }}
      >
        <SelectTrigger size="sm" className="w-36" aria-label="Clip keyframes">
          <SelectValue>{(v: VideoChoice) => videoLabel(v)}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {VIDEO_CHOICES.map((k) => (
              <SelectItem key={k} value={k}>
                {videoLabel(k)}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
      <Select
        value={audioEnvelope}
        onValueChange={(v) => {
          if (v) useTimeline.getState().setAudioEnvelope(v);
        }}
      >
        <SelectTrigger size="sm" className="w-28" aria-label="Sound keyframes">
          <SelectValue>
            {(v: (typeof AUDIO_ENVELOPES)[number]) => ENVELOPE_INFO[v].label}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {AUDIO_ENVELOPES.map((k) => (
              <SelectItem key={k} value={k}>
                {ENVELOPE_INFO[k].label}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </>
  );
}

function RangeBadge({ fps, duration }: { fps: number; duration: number }) {
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);
  const rulerMode = useTimeline((s) => s.rulerMode);
  if (inPoint === null && outPoint === null) return null;
  return (
    <Badge variant="outline" className="gap-1 tabular-nums">
      {formatTime(inPoint ?? 0, fps, rulerMode)}–{formatTime(outPoint ?? duration, fps, rulerMode)}
      <button
        type="button"
        aria-label="Clear the loop"
        onClick={() => usePlayback.getState().clearRange()}
      >
        <X className="size-3" />
      </button>
    </Badge>
  );
}

function Clock({ fps, duration }: { fps: number; duration: number }) {
  const frame = usePlayback((s) => s.frame);
  const rulerMode = useTimeline((s) => s.rulerMode);
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground px-1 text-xs whitespace-nowrap tabular-nums"
            onClick={() =>
              useTimeline.getState().setRulerMode(rulerMode === "seconds" ? "timecode" : "seconds")
            }
          />
        }
      >
        {formatTime(frame, fps, rulerMode)} / {formatTime(duration, fps, rulerMode)}
      </TooltipTrigger>
      <TooltipContent>Switch between seconds and timecode</TooltipContent>
    </Tooltip>
  );
}

function SourceBadges({
  workflow,
  sourceNodeId,
}: {
  workflow: Workflow;
  sourceNodeId: string | null;
}) {
  const source = workflow.nodes.find((n) => n.id === sourceNodeId);
  const output = workflow.nodes.find((n) => isOutputType(n.type) && !n.disabled);
  const showingExport = !!source && isOutputType(source.type);
  return (
    <>
      {source ? (
        <Badge variant="secondary" className="max-w-40 truncate">
          {nodeName(source)}
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
    </>
  );
}

export function TimelineToolbar({
  actions,
  workflow,
  sourceNodeId,
  fps,
  duration,
}: {
  actions: React.RefObject<TimelineActions | null>;
  workflow: Workflow;
  sourceNodeId: string | null;
  fps: number;
  duration: number;
}) {
  return (
    <div className="flex h-9 shrink-0 items-center gap-1 overflow-x-auto border-b px-2">
      <ToolPicker />
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <IconTip
        label="Split at the playhead"
        tip={
          <>
            Split at the playhead <Kbd>S</Kbd>
          </>
        }
        onClick={() => actions.current?.split()}
      >
        <Scissors />
      </IconTip>
      <IconTip
        label="Add a marker"
        tip={
          <>
            Add a marker <Kbd>M</Kbd>
          </>
        }
        onClick={() => actions.current?.addMarker()}
      >
        <Bookmark />
      </IconTip>
      <EnvelopePickers />
      <div className="flex-1" />
      <SourceBadges workflow={workflow} sourceNodeId={sourceNodeId} />
      <RangeBadge fps={fps} duration={duration} />
      <Clock fps={fps} duration={duration} />
      <IconTip
        label="Zoom out"
        tip={
          <>
            Zoom out <Kbd>{shortcutLabel("CmdOrCtrl+−")}</Kbd>
          </>
        }
        onClick={() => actions.current?.zoomBy(1 / ZOOM_STEP)}
      >
        <ZoomOut />
      </IconTip>
      <IconTip
        tip={
          <>
            Show the whole film <Kbd>Z</Kbd>
          </>
        }
        onClick={() => actions.current?.fit()}
      >
        Fit
      </IconTip>
      <IconTip
        label="Zoom in"
        tip={
          <>
            Zoom in <Kbd>{shortcutLabel("CmdOrCtrl+=")}</Kbd>
          </>
        }
        onClick={() => actions.current?.zoomBy(ZOOM_STEP)}
      >
        <ZoomIn />
      </IconTip>
    </div>
  );
}
