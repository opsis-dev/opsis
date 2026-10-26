import { findById } from "@opsis/video/common/lists";
import { atLeastOneFrame } from "@opsis/video/common/time";
import { Clapperboard } from "lucide-react";
import { useMemo, useState } from "react";
import { exportReframe, variantSize } from "@/core/nodes";
import type { TimelineSpec } from "@/generated/bindings";
import { nodeName } from "@/store/catalog";
import { EmptyPanel } from "../app/empty-panel";
import { useProjectFiles } from "../store/project-files";
import { useViewOptions } from "../store/view-options";
import { useWorkflowStore } from "../store/workflow";
import { useFilm } from "./film";
import { PreviewStage, togglePlayback } from "./preview-controls";
import { usePreviewSound, useStreamedPreview } from "./preview-stream";
import { PreviewToolbar } from "./preview-toolbar";
import { useCompiledContext } from "./use-compiled";

function NothingToPreview() {
  return (
    <EmptyPanel
      icon={Clapperboard}
      title="Nothing to preview yet"
      description="Add a clip-producing node — a title, an element, or a recording through Auto zoom."
    />
  );
}

function usePreviewSpec(): TimelineSpec | null {
  const film = useFilm();
  const workflow = useWorkflowStore((s) => s.workflow);
  const quality = useViewOptions((s) => s.previewQuality);
  const shape = useViewOptions((s) => s.previewShape);
  const versions = useProjectFiles((s) => s.versions);
  return useMemo(() => {
    if (!film.spec || !workflow) return null;
    const base = { ...film.spec, assets: { versions }, quality };
    return shape === "project"
      ? base
      : {
          ...base,
          ...variantSize(shape, workflow.settings),
          reframe: exportReframe(workflow.nodes),
        };
  }, [film.spec, quality, shape, workflow, versions]);
}

export function PreviewDock() {
  const { preview } = useCompiledContext();
  const film = useFilm();
  const spec = usePreviewSpec();
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const [stage, setStage] = useState<HTMLDivElement | null>(null);
  const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
  const sound = usePreviewSound(film.stem);
  const sourceNodeId = preview?.sourceNodeId ?? null;
  const source = useWorkflowStore((s) => findById(s.workflow?.nodes, sourceNodeId));
  useStreamedPreview({ projectDir, preview, spec, stage, canvas });

  if (!spec) return <NothingToPreview />;

  const duration = atLeastOneFrame(spec.durationInFrames);
  return (
    <div className="flex h-full flex-col gap-2 p-3">
      <PreviewToolbar
        spec={spec}
        sourceTitle={source ? nodeName(source) : null}
        stale={film.stale}
      />
      <PreviewStage sound={sound} fps={spec.fps} duration={duration} onStage={setStage}>
        <canvas
          ref={setCanvas}
          aria-label="Preview"
          onClick={() => togglePlayback(sound)}
          className="size-full object-contain"
          style={{ aspectRatio: `${spec.width} / ${spec.height}` }}
        />
      </PreviewStage>
    </div>
  );
}
