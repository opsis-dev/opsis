import { Player, type PlayerRef } from "@remotion/player";
import { Clapperboard } from "lucide-react";
import { useEffect, useMemo, useRef } from "react";
import { getNodeDef } from "@/shared/nodes/registry";
import type { TimelineSpec } from "@/shared/schema/timeline";
import { DemoTimeline } from "@/video/demo-timeline";
import { Badge } from "@/renderer/components/ui/badge";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useUiStore, type PreviewQuality } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { useCompiledContext } from "./use-compiled";
import { useStillsStore } from "../store/stills";

export function PreviewDock() {
  const { preview } = useCompiledContext();
  const workflow = useWorkflowStore((s) => s.workflow);
  const selected = useUiStore((s) => s.selectedIds);
  const setActive = useUiStore((s) => s.setActive);
  const player = useRef<PlayerRef>(null);
  const quality = useUiStore((s) => s.previewQuality);
  const setQuality = useUiStore((s) => s.setPreviewQuality);

  const spec: TimelineSpec | null = useMemo(
    () =>
      preview
        ? { ...preview.spec, assets: { projectBase: "demoflow-file://project/" }, quality }
        : null,
    [preview, quality],
  );

  const sourceNodeId = preview?.sourceNodeId ?? null;
  useEffect(() => {
    if (sourceNodeId) useStillsStore.getState().want([sourceNodeId]);
  }, [sourceNodeId]);

  const selectedId = selected.length === 1 ? selected[0] : null;
  useEffect(() => {
    if (!selectedId || !preview) return;
    const span = preview.spans[selectedId];
    if (span) player.current?.seekTo(span.from);
  }, [selectedId, preview]);

  useEffect(() => {
    const p = player.current;
    if (!p || !preview) return;
    const update = ({ detail }: { detail: { frame: number } }) => {
      const ids = Object.entries(preview.spans)
        .filter(([, s]) => detail.frame >= s.from && detail.frame < s.to)
        .map(([id]) => id)
        .sort();
      setActive(ids);
    };
    p.addEventListener("frameupdate", update);
    return () => p.removeEventListener("frameupdate", update);
  }, [preview, setActive]);

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

  return (
    <div className="flex h-full flex-col gap-2 p-3">
      <div className="text-muted-foreground flex items-center gap-2 text-xs">
        {sourceTitle ? <Badge variant="secondary">{sourceTitle}</Badge> : null}
        <span className="tabular-nums">
          {spec.width}×{spec.height} · {spec.fps} fps ·{" "}
          {(spec.durationInFrames / spec.fps).toFixed(1)}s
        </span>
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
          ref={player}
          component={DemoTimeline}
          inputProps={spec}
          durationInFrames={Math.max(1, spec.durationInFrames)}
          fps={spec.fps}
          compositionWidth={spec.width}
          compositionHeight={spec.height}
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
