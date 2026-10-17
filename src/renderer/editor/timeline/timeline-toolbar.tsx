import { Bookmark, Magnet, Scissors, X, ZoomIn, ZoomOut } from "lucide-react";
import { nodeName } from "@/shared/nodes/registry";
import { isOutputType } from "@/shared/nodes/output";
import {
  AUDIO_ENVELOPES,
  VISUAL_ENVELOPES,
  type VisualEnvelopeKey,
} from "@/shared/schema/timeline";
import type { Workflow } from "@/shared/schema/workflow";
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
import { useFormatTime, useLoopRange, useTimeline } from "./timeline-state";
import { COMMAND_KEYS, TOOLS, ZOOM_STEP } from "./timeline-tools";
import type { TimelineCommands } from "./use-timeline-commands";
import { findById } from "@/shared/lists";

type VideoChoice = VisualEnvelopeKey | "off";

const VIDEO_CHOICES: VideoChoice[] = ["off", ...VISUAL_ENVELOPES];

const videoLabel = (v: VideoChoice) => (v === "off" ? "No keyframes" : ENVELOPE_INFO[v].label);

function IconTip({
  label,
  iconOnly = true,
  tip,
  onClick,
  children,
}: {
  label: string;
  iconOnly?: boolean;
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
            size={iconOnly ? "icon-sm" : "sm"}
            className={iconOnly ? undefined : "px-2"}
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
          const next = findById(TOOLS, v[0]);
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
  const { inPoint, outPoint } = useLoopRange();
  const timeAt = useFormatTime(fps);
  if (inPoint === null && outPoint === null) return null;
  return (
    <Badge variant="outline" className="gap-1 tabular-nums">
      {timeAt(inPoint ?? 0)}–{timeAt(outPoint ?? duration)}
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
  const timeAt = useFormatTime(fps);
  const switchMode = () => {
    const { rulerMode, setRulerMode } = useTimeline.getState();
    setRulerMode(rulerMode === "seconds" ? "timecode" : "seconds");
  };
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground px-1 text-xs whitespace-nowrap tabular-nums"
            onClick={switchMode}
          />
        }
      >
        {timeAt(frame)} / {timeAt(duration)}
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
  const source = findById(workflow.nodes, sourceNodeId);
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
  commands,
  workflow,
  sourceNodeId,
  fps,
  duration,
}: {
  commands: TimelineCommands;
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
            Split at the playhead <Kbd>{COMMAND_KEYS.split}</Kbd>
          </>
        }
        onClick={commands.split}
      >
        <Scissors />
      </IconTip>
      <IconTip
        label="Add a marker"
        tip={
          <>
            Add a marker <Kbd>{COMMAND_KEYS.addMarker}</Kbd>
          </>
        }
        onClick={commands.addMarker}
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
        onClick={() => commands.zoomBy(1 / ZOOM_STEP)}
      >
        <ZoomOut />
      </IconTip>
      <IconTip
        label="Fit the whole film"
        iconOnly={false}
        tip={
          <>
            Show the whole film <Kbd>{COMMAND_KEYS.fit}</Kbd>
          </>
        }
        onClick={commands.fit}
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
        onClick={() => commands.zoomBy(ZOOM_STEP)}
      >
        <ZoomIn />
      </IconTip>
    </div>
  );
}
