import { findById } from "@opsis/video/common/lists";
import { Bookmark, Magnet, Pause, Play, Scissors, X, ZoomIn, ZoomOut } from "lucide-react";
import { shortcutLabel } from "@/app/menu-shortcuts";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Separator } from "@/components/ui/separator";
import { Toggle } from "@/components/ui/toggle";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { EnvelopeKey, Workflow } from "@/generated/bindings";
import { isOutputType, nodeName, timelineBook } from "@/store/catalog";
import { usePlayback } from "@/store/playback";
import { useUiStore } from "@/store/ui";
import { envelopeTrack } from "@/timeline/envelopes";
import { OptionSelect } from "../option-select";
import { WithTooltip } from "../with-tooltip";
import { useFormatTime, useLoopRange, useTimeline } from "./timeline-state";
import { COMMAND_KEYS, TOOLS, ZOOM_KEYS, ZOOM_STEP } from "./timeline-tools";
import type { TimelineCommands } from "./use-timeline-commands";

type VideoChoice = EnvelopeKey | "off";

function tracksFor(audio: boolean): EnvelopeKey[] {
  return timelineBook()
    .envelopes.filter((t) => t.audio === audio)
    .map((t) => t.key);
}

function videoLabel(v: VideoChoice) {
  return v === "off" ? "No keyframes" : envelopeTrack(v).label;
}

function IconTip({
  label,
  shortcut,
  iconOnly = true,
  onClick,
  children,
}: {
  label: string;
  shortcut: string;
  iconOnly?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <WithTooltip
      trigger={
        <Button
          variant="ghost"
          size={iconOnly ? "icon-sm" : "sm"}
          className={iconOnly ? undefined : "px-2"}
          aria-label={label}
          onClick={onClick}
        />
      }
      tip={
        <>
          {label} <Kbd>{shortcut}</Kbd>
        </>
      }
    >
      {children}
    </WithTooltip>
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
          <WithTooltip
            key={t.id}
            trigger={<ToggleGroupItem value={t.id} aria-label={t.label} />}
            tipClassName="max-w-64"
            tip={
              <>
                {t.label} <Kbd>{t.key}</Kbd>
                <div className="text-muted-foreground mt-1">{t.help}</div>
              </>
            }
          >
            <t.icon />
          </WithTooltip>
        ))}
      </ToggleGroup>
      <WithTooltip
        trigger={
          <Toggle
            size="sm"
            aria-label="Snap"
            pressed={snapping}
            onPressedChange={(v) => useTimeline.getState().setSnapping(v)}
          />
        }
        tip={
          <>
            Snap to the playhead, clip edges, markers, actions and beats. Hold <Kbd>Alt</Kbd> to
            turn it off while dragging.
          </>
        }
      >
        <Magnet />
      </WithTooltip>
    </>
  );
}

function EnvelopePickers() {
  const videoChoice: VideoChoice = useTimeline((s) => s.videoEnvelope) ?? "off";
  const audioEnvelope = useTimeline((s) => s.audioEnvelope);
  return (
    <>
      <OptionSelect
        label="Clip keyframes"
        value={videoChoice}
        options={["off", ...tracksFor(false)]}
        optionLabel={videoLabel}
        className="w-36"
        onChange={(v) => useTimeline.getState().setVideoEnvelope(v === "off" ? null : v)}
      />
      <OptionSelect
        label="Sound keyframes"
        value={audioEnvelope}
        options={tracksFor(true)}
        optionLabel={(v) => envelopeTrack(v).label}
        className="w-28"
        onChange={(v) => useTimeline.getState().setAudioEnvelope(v)}
      />
    </>
  );
}

function RangeBadge({ fps, duration }: { fps: number; duration: number }) {
  const loop = useLoopRange(duration);
  const timeAt = useFormatTime(fps);
  if (!loop) return null;
  return (
    <Badge variant="outline" className="gap-1 tabular-nums">
      {timeAt(loop.from)}–{timeAt(loop.to)}
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

function PlayButton() {
  const playing = usePlayback((s) => s.playing);
  return (
    <IconTip
      label={playing ? "Pause" : "Play"}
      shortcut="Space"
      onClick={() => usePlayback.getState().toggle()}
    >
      {playing ? <Pause /> : <Play />}
    </IconTip>
  );
}

function Clock({ fps, duration }: { fps: number; duration: number }) {
  const frame = usePlayback((s) => s.frame);
  const timeAt = useFormatTime(fps);
  function switchMode() {
    const { rulerMode, setRulerMode } = useTimeline.getState();
    setRulerMode(rulerMode === "seconds" ? "timecode" : "seconds");
  }
  return (
    <WithTooltip
      trigger={
        <button
          type="button"
          className="text-muted-foreground hover:text-foreground px-1 text-xs whitespace-nowrap tabular-nums"
          onClick={switchMode}
        />
      }
      tip="Switch between seconds and timecode"
    >
      {timeAt(frame)} / {timeAt(duration)}
    </WithTooltip>
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
      <IconTip label="Split at the playhead" shortcut={COMMAND_KEYS.split} onClick={commands.split}>
        <Scissors />
      </IconTip>
      <IconTip label="Add a marker" shortcut={COMMAND_KEYS.addMarker} onClick={commands.addMarker}>
        <Bookmark />
      </IconTip>
      <EnvelopePickers />
      <div className="flex-1" />
      <SourceBadges workflow={workflow} sourceNodeId={sourceNodeId} />
      <RangeBadge fps={fps} duration={duration} />
      <PlayButton />
      <Clock fps={fps} duration={duration} />
      <IconTip
        label="Zoom out"
        shortcut={shortcutLabel(`CmdOrCtrl+${ZOOM_KEYS.out.label}`)}
        onClick={() => commands.zoomBy(1 / ZOOM_STEP)}
      >
        <ZoomOut />
      </IconTip>
      <IconTip
        label="Show the whole film"
        shortcut={COMMAND_KEYS.fit}
        iconOnly={false}
        onClick={commands.fit}
      >
        Fit
      </IconTip>
      <IconTip
        label="Zoom in"
        shortcut={shortcutLabel(`CmdOrCtrl+${ZOOM_KEYS.in.label}`)}
        onClick={() => commands.zoomBy(ZOOM_STEP)}
      >
        <ZoomIn />
      </IconTip>
    </div>
  );
}
