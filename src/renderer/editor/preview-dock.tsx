import { Player, type PlayerRef } from "@remotion/player";
import { Clapperboard } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { getNodeDef } from "@/shared/nodes/registry";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { Badge } from "@/renderer/components/ui/badge";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { Spinner } from "@/renderer/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useAudioPreview } from "../store/audio-preview";
import { registerPlayer, usePlayback } from "../store/playback";
import { useUiStore, type PreviewQuality } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { useStillsStore } from "../store/stills";
import { PreviewComposition } from "./audio-insert";
import { useFilm } from "./film";
import { useCompiledContext } from "./use-compiled";

export function PreviewDock() {
  const { preview } = useCompiledContext();
  const film = useFilm();
  const workflow = useWorkflowStore((s) => s.workflow);
  const selected = useUiStore((s) => s.selectedIds);
  const pinned = useUiStore((s) => s.pinnedSource);
  const setActive = useUiStore((s) => s.setActive);
  const [player, setPlayer] = useState<PlayerRef | null>(null);
  const quality = useUiStore((s) => s.previewQuality);
  const setQuality = useUiStore((s) => s.setPreviewQuality);
  const rate = usePlayback((s) => s.rate);
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);

  const spec: TimelineSpec | null = useMemo(
    () =>
      film.audible
        ? { ...film.audible, assets: { projectBase: "demoflow-file://project/" }, quality }
        : null,
    [film.audible, quality],
  );

  useEffect(() => {
    if (!film.stale) useAudioPreview.getState().clear();
  }, [film.stale]);

  const sourceNodeId = preview?.sourceNodeId ?? null;
  useEffect(() => {
    if (sourceNodeId) useStillsStore.getState().want([sourceNodeId]);
  }, [sourceNodeId]);

  const selectedId = selected.length === 1 && !pinned ? selected[0] : null;
  useEffect(() => {
    if (!selectedId || !preview) return;
    const span = preview.spans[selectedId];
    if (span) player?.seekTo(span.from);
  }, [selectedId, preview, player]);

  const duration = spec ? Math.max(1, spec.durationInFrames) : 1;
  const fps = spec?.fps ?? 30;
  useEffect(() => {
    usePlayback.getState().report({ fps, durationInFrames: duration });
  }, [fps, duration]);

  useEffect(() => {
    if (!player) return;
    registerPlayer({
      seek: (frame) => player.seekTo(frame),
      play: () => player.play(),
      pause: () => player.pause(),
      setRate: (r) => usePlayback.getState().report({ rate: r }),
    });
    const frame = usePlayback.getState().frame;
    if (frame > 0) player.seekTo(Math.min(frame, duration - 1));
    return () => {
      registerPlayer(null);
      usePlayback.getState().report({ playing: false });
    };
  }, [player, duration]);

  useEffect(() => {
    const p = player;
    if (!p || !preview) return;
    const update = ({ detail }: { detail: { frame: number } }) => {
      usePlayback.getState().report({ frame: detail.frame });
      const ids = Object.entries(preview.spans)
        .filter(([, s]) => detail.frame >= s.from && detail.frame < s.to)
        .map(([id]) => id)
        .sort();
      setActive(ids);
    };
    const onPlay = () => usePlayback.getState().report({ playing: true });
    const onPause = () => usePlayback.getState().report({ playing: false });
    p.addEventListener("frameupdate", update);
    p.addEventListener("seeked", update);
    p.addEventListener("play", onPlay);
    p.addEventListener("pause", onPause);
    return () => {
      p.removeEventListener("frameupdate", update);
      p.removeEventListener("seeked", update);
      p.removeEventListener("play", onPlay);
      p.removeEventListener("pause", onPause);
    };
  }, [preview, setActive, player]);

  if (!spec || !workflow) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Clapperboard />
          </EmptyMedia>
          <EmptyTitle>Nothing to preview yet</EmptyTitle>
          <EmptyDescription>
            Add a clip-producing node — a title, an element, or a recording through Auto zoom.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  const source = preview?.sourceNodeId
    ? workflow.nodes.find((n) => n.id === preview.sourceNodeId)
    : undefined;
  const sourceTitle = source
    ? (source.label ?? getNodeDef(source.type)?.title ?? source.type)
    : null;
  const range =
    inPoint !== null || outPoint !== null
      ? {
          inFrame: Math.min(duration - 2, Math.max(0, inPoint ?? 0)),
          outFrame: Math.min(duration - 1, Math.max((inPoint ?? 0) + 1, outPoint ?? duration - 1)),
        }
      : {};

  return (
    <div className="flex h-full flex-col gap-2 p-3">
      <div className="text-muted-foreground flex items-center gap-2 text-xs">
        {sourceTitle ? <Badge variant="secondary">{sourceTitle}</Badge> : null}
        <span className="truncate tabular-nums">
          {spec.width}×{spec.height} · {spec.fps} fps ·{" "}
          {(spec.durationInFrames / spec.fps).toFixed(1)}s
        </span>
        {film.stale ? (
          <Tooltip>
            <TooltipTrigger render={<span className="flex shrink-0 items-center gap-1" />}>
              <Spinner className="size-3" />
              Updating sound
            </TooltipTrigger>
            <TooltipContent>
              The sound is being mixed again. You hear the previous mix until it's ready.
            </TooltipContent>
          </Tooltip>
        ) : null}
        <div className="flex-1" />
        <ToggleGroup
          variant="outline"
          size="sm"
          value={[quality]}
          onValueChange={(v) => v[0] && setQuality(v[0] as PreviewQuality)}
        >
          <Tooltip>
            <TooltipTrigger render={<ToggleGroupItem value="fast" />}>Fast</TooltipTrigger>
            <TooltipContent>
              Smooth playback: smaller recordings and no motion blur. The export is unchanged.
            </TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger render={<ToggleGroupItem value="exact" />}>Exact</TooltipTrigger>
            <TooltipContent>Exactly what the export looks like. May stutter.</TooltipContent>
          </Tooltip>
        </ToggleGroup>
      </div>
      <div className="grid min-h-0 flex-1 place-items-center">
        <Player
          ref={setPlayer}
          component={PreviewComposition}
          inputProps={spec}
          durationInFrames={duration}
          fps={spec.fps}
          compositionWidth={spec.width}
          compositionHeight={spec.height}
          playbackRate={rate}
          {...range}
          controls
          loop
          clickToPlay
          acknowledgeRemotionLicense
          style={{ width: "100%", height: "100%", maxHeight: "100%" }}
        />
      </div>
    </div>
  );
}
