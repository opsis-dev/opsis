import { Player, type PlayerRef } from "@remotion/player";
import { Clapperboard } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { exportReframe, variantSize } from "@/shared/nodes/output";
import { nodeName } from "@/shared/nodes/registry";
import type { TimelineSpec } from "@/shared/schema/timeline";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/renderer/components/ui/empty";
import { playRange, usePlayback } from "../store/playback";
import { useStillsStore } from "../store/stills";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { PreviewComposition } from "./audio-insert";
import { useFilm } from "./film";
import { PreviewToolbar } from "./preview-toolbar";
import { useCompiledContext } from "./use-compiled";
import { usePlayerBridge, useSeekToSelection } from "./use-player-bridge";

const PROJECT_BASE = "demoflow-file://project/";

function NothingToPreview() {
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

function usePreviewSpec(): TimelineSpec | null {
  const film = useFilm();
  const workflow = useWorkflowStore((s) => s.workflow);
  const quality = useUiStore((s) => s.previewQuality);
  const shape = useUiStore((s) => s.previewShape);
  return useMemo(() => {
    if (!film.audible || !workflow) return null;
    const base = { ...film.audible, assets: { projectBase: PROJECT_BASE }, quality };
    return shape === "project"
      ? base
      : {
          ...base,
          ...variantSize(shape, workflow.settings),
          reframe: exportReframe(workflow.nodes),
        };
  }, [film.audible, quality, shape, workflow]);
}

export function PreviewDock() {
  const { preview } = useCompiledContext();
  const film = useFilm();
  const spec = usePreviewSpec();
  const [player, setPlayer] = useState<PlayerRef | null>(null);
  const rate = usePlayback((s) => s.rate);
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);
  const sourceNodeId = preview?.sourceNodeId ?? null;
  const source = useWorkflowStore((s) =>
    sourceNodeId ? s.workflow?.nodes.find((n) => n.id === sourceNodeId) : undefined,
  );

  useEffect(() => {
    if (sourceNodeId) useStillsStore.getState().want([sourceNodeId]);
  }, [sourceNodeId]);

  const duration = spec ? Math.max(1, spec.durationInFrames) : 1;
  useSeekToSelection(player, preview);
  usePlayerBridge(player, preview, { fps: spec?.fps ?? 30, durationInFrames: duration });

  if (!spec) return <NothingToPreview />;

  return (
    <div className="flex h-full flex-col gap-2 p-3">
      <PreviewToolbar
        spec={spec}
        sourceTitle={source ? nodeName(source) : null}
        stale={film.stale}
      />
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
          {...playRange(inPoint, outPoint, duration)}
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
