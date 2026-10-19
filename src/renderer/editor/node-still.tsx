import { Thumbnail } from "@remotion/player";
import { useStore as useFlowStore } from "@xyflow/react";
import { useEffect, useMemo, useRef, useState, type RefObject } from "react";
import { useSettledAfterKeyChange } from "@/renderer/lib/use-settled-after-key-change";
import { PROJECT_FILE_BASE, projectRef } from "@/shared/asset-ref";
import { clipToSpec, isClip } from "@/shared/compile/clip";
import { errorMessage } from "@/shared/errors";
import { stableJson } from "@/shared/hash";
import { showsClipStill, type NodeDef } from "@/shared/nodes/define";
import { PROJECT_DIRS } from "@/shared/project-dirs";
import type { TimelineSpec } from "@/shared/schema/timeline";
import type { ClipValue, NodeResult } from "@/shared/schema/values";
import type { WorkflowSettings } from "@/shared/schema/workflow";
import { atLeastOneFrame, clampFrame } from "@/shared/timeline/time";
import { DemoTimeline } from "@/video/demo-timeline";
import { useEnvironment } from "../store/environment";
import { useRunStore } from "../store/run";
import { useStillsStore } from "../store/stills";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import { requireWorkflow } from "./require-workflow";
import { useThumbSlot } from "./thumb-slots";
import { useCompiledContext } from "./use-compiled";

const LIVE_THUMB_MIN_ZOOM = 0.55;
const VIDEO_ASPECT = 16 / 9;
const SETTLE_MS = 400;
const OBSERVE_MARGIN = "120px";

type SettledClip = { key: string; clip: ClipValue; settings: WorkflowSettings };

const fastSpec = ({ clip, settings }: SettledClip): TimelineSpec => ({
  ...clipToSpec(clip, settings),
  quality: "fast",
});

const middleFrame = (spec: TimelineSpec): number =>
  clampFrame(spec.durationInFrames / 2, spec.durationInFrames);

function posterOf(result: NodeResult | undefined): string | null {
  for (const a of result?.artifacts ?? []) {
    if (a.poster) return a.poster;
    if (a.kind === "image") return a.path;
  }
  return null;
}

function useSettledClip(nodeId: string): SettledClip | null {
  const { result: compiled } = useCompiledContext();
  const settings = useWorkflowStore((s) => requireWorkflow(s).settings);
  const clip = compiled?.values[nodeId]?.clip;
  const current = isClip(clip) ? { key: stableJson([clip, settings]), clip, settings } : null;
  return useSettledAfterKeyChange(current, current?.key ?? "", SETTLE_MS);
}

function useOnScreen<T extends Element>(): [RefObject<T | null>, boolean] {
  const ref = useRef<T>(null);
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(([entry]) => setVisible(entry.isIntersecting), {
      rootMargin: OBSERVE_MARGIN,
    });
    io.observe(el);
    return () => io.disconnect();
  }, []);
  return [ref, visible];
}

function FallbackThumbnail({ nodeId }: { nodeId: string }) {
  const settings = useWorkflowStore((s) => requireWorkflow(s).settings);
  const zoom = useFlowStore((s) => s.transform[2]);
  const [ref, visible] = useOnScreen<HTMLDivElement>();
  const settled = useSettledClip(nodeId);
  const holding = useThumbSlot(nodeId, visible && zoom >= LIVE_THUMB_MIN_ZOOM && !!settled);
  const spec = useMemo(
    () => (settled ? { ...fastSpec(settled), assets: { projectBase: PROJECT_FILE_BASE } } : null),
    [settled],
  );

  if (!spec) return null;
  const aspectRatio = `${settings.width} / ${settings.height}`;
  const fittedSide = settings.width / settings.height >= VIDEO_ASPECT ? "width" : "height";
  return (
    <div
      ref={ref}
      className="bg-muted/40 mx-3 flex aspect-video items-center justify-center overflow-hidden rounded-md"
    >
      {holding ? (
        <Thumbnail
          component={DemoTimeline}
          inputProps={spec}
          compositionWidth={spec.width}
          compositionHeight={spec.height}
          durationInFrames={atLeastOneFrame(spec.durationInFrames)}
          fps={spec.fps}
          frameToDisplay={middleFrame(spec)}
          style={{ aspectRatio, [fittedSide]: "100%" }}
          errorFallback={() => null}
        />
      ) : null}
    </div>
  );
}

function FramePicture({ src, lazy }: { src: string; lazy?: boolean }) {
  return (
    <div className="mx-3 overflow-hidden rounded-md bg-black">
      <img
        src={fileUrl(src)}
        alt=""
        loading={lazy ? "lazy" : undefined}
        draggable={false}
        className="block aspect-video w-full object-contain"
      />
    </div>
  );
}

export function NodeThumbnail({ nodeId, def }: { nodeId: string; def: NodeDef }) {
  const result = useRunStore((s) => s.results[nodeId]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const poster = posterOf(result);
  if (poster && projectDir) return <FramePicture src={poster} lazy />;
  return showsClipStill(def) ? <CardStill nodeId={nodeId} /> : null;
}

function CardStill({ nodeId }: { nodeId: string }) {
  const engine = useEnvironment((s) => s.env?.engine.complete);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const hash = useStillsStore((s) => s.index[nodeId]);
  const wanted = useStillsStore((s) => !!s.wanted[nodeId]);
  const selected = useUiStore((s) => s.selectedIds.includes(nodeId));
  const settled = useSettledClip(nodeId);

  useEffect(() => {
    if (!engine || !projectDir || !settled || (!hash && !wanted)) return;
    const spec = fastSpec(settled);
    let current = true;
    void window.opsis
      .requestStill({
        projectDir,
        nodeId,
        key: settled.key,
        spec,
        frame: middleFrame(spec),
        priority: selected || !hash,
      })
      .then((r) => {
        if (current && r.ready) useStillsStore.getState().ready(projectDir, nodeId, r.hash);
      })
      .catch((error: unknown) =>
        console.warn(`Couldn't make the card picture for ${nodeId}: ${errorMessage(error)}`),
      );
    return () => {
      current = false;
    };
  }, [engine, projectDir, settled, hash, wanted, selected, nodeId]);

  if (engine === false) return <FallbackThumbnail nodeId={nodeId} />;
  if (!hash) return null;
  return <FramePicture src={projectRef(`${PROJECT_DIRS.stills}/${hash}.jpg`)} />;
}
