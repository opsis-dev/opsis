import { Thumbnail } from "@remotion/player";
import { useStore as useFlowStore } from "@xyflow/react";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { create } from "zustand";
import { useShallow } from "zustand/react/shallow";
import { clipToSpec, isClip } from "@/shared/compile/clip";
import { workflowVars } from "@/shared/compile/compile";
import type { CardRow } from "@/shared/nodes/card";
import type { NodeDef } from "@/shared/nodes/define";
import { defaultParams } from "@/shared/nodes/registry";
import type { TimelineSpec } from "@/shared/schema/timeline";
import type { ClipValue, NodeResult } from "@/shared/schema/values";
import type { WorkflowNode, WorkflowSettings } from "@/shared/schema/workflow";
import { interpolate } from "@/shared/vars";
import { DemoTimeline } from "@/video/demo-timeline";
import { cn } from "@/renderer/lib/utils";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useStillsStore } from "../store/stills";
import { useEnvironment } from "../store/environment";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import { NodeIcon } from "./icons";
import { useCompiledContext } from "./use-compiled";
import { stableJson } from "@/shared/hash";
import { errorMessage } from "@/shared/errors";
import { PROJECT_FILE_BASE, projectRef } from "@/shared/asset-ref";
import { PROJECT_DIRS } from "@/shared/project-dirs";
import { frameIn } from "@/shared/timeline/time";

const MAX_LIVE_THUMBS = 6;
const MIN_ZOOM = 0.55;
const MAX_ROWS = 4;
const SETTLE_MS = 400;
const OBSERVE_MARGIN = "120px";

const useThumbSlots = create<{
  active: string[];
  claim: (id: string, priority: boolean) => boolean;
  release: (id: string) => void;
}>()((set, get) => ({
  active: [],
  claim: (id, priority) => {
    const { active } = get();
    if (active.includes(id)) return true;
    if (active.length < MAX_LIVE_THUMBS) {
      set({ active: [...active, id] });
      return true;
    }
    if (!priority) return false;
    set({ active: [...active.slice(1), id] });
    return true;
  },
  release: (id) => set((s) => ({ active: s.active.filter((a) => a !== id) })),
}));

function useCardInputs(node: WorkflowNode) {
  const variables = useWorkflowStore((s) => s.workflow?.variables);
  const name = useWorkflowStore((s) => s.workflow?.name ?? "");
  const settings = useWorkflowStore((s) => s.workflow?.settings);
  const incoming = useWorkflowStore(
    useShallow((s) => (s.workflow?.edges ?? []).filter((e) => e.target === node.id)),
  );
  const result = useRunStore((s) => s.results[node.id]);
  const secrets = useSecretsStore((s) => s.names);
  const { result: compiled } = useCompiledContext();
  const clip = compiled?.values[node.id]?.clip;
  const clipFrames = isClip(clip) ? clip.durationInFrames : undefined;
  const connected = useMemo(() => incoming.map((e) => e.targetHandle), [incoming]);
  const inputFrames = useMemo(() => {
    const out: Record<string, number> = {};
    for (const e of incoming) {
      const v = compiled?.values[e.source]?.[e.sourceHandle];
      if (isClip(v)) out[e.targetHandle] = v.durationInFrames;
    }
    return out;
  }, [incoming, compiled]);
  return { variables, name, settings, result, secrets, connected, clipFrames, inputFrames };
}

export function useCardRows(node: WorkflowNode, def: NodeDef): CardRow[] {
  const { variables, name, settings, result, secrets, connected, clipFrames, inputFrames } =
    useCardInputs(node);
  return useMemo(() => {
    if (!def.card || !settings || !variables) return [];
    const raw = node.params;
    const params = {
      ...defaultParams(node.type),
      ...interpolate(raw, workflowVars({ name, variables })),
    };
    try {
      return def
        .card({
          params,
          raw,
          settings,
          result,
          secrets: secrets ?? undefined,
          connected,
          clipFrames,
          inputFrames,
        })
        .filter((r): r is CardRow => Boolean(r))
        .slice(0, MAX_ROWS);
    } catch (error) {
      return [
        {
          icon: "triangle-alert",
          text: "Couldn't show this node's summary",
          title: errorMessage(error),
          tone: "warn",
        },
      ];
    }
  }, [
    def,
    node.type,
    node.params,
    variables,
    name,
    settings,
    result,
    secrets,
    connected,
    clipFrames,
    inputFrames,
  ]);
}

export function CardRows({ rows }: { rows: CardRow[] }) {
  if (!rows.length) return null;
  return (
    <ul className="flex flex-col gap-1 px-3">
      {rows.map((r, i) => (
        <li
          key={i}
          title={r.title ?? r.text}
          className={cn(
            "flex min-w-0 items-center gap-1.5 text-xs",
            r.tone === "warn"
              ? "text-warning"
              : r.tone === "muted"
                ? "text-muted-foreground"
                : "text-foreground/85",
          )}
        >
          <NodeIcon
            name={r.icon}
            className={cn("size-3.5 shrink-0", r.tone ? undefined : "text-muted-foreground")}
          />
          <span className="truncate">{r.text}</span>
        </li>
      ))}
    </ul>
  );
}

function posterOf(result: NodeResult | undefined): string | null {
  for (const a of result?.artifacts ?? []) {
    if (a.poster) return a.poster;
    if (a.kind === "image") return a.path;
  }
  return null;
}

type SettledClip = { key: string; clip: ClipValue; settings: WorkflowSettings };

const fastSpec = ({ clip, settings }: SettledClip): TimelineSpec => ({
  ...clipToSpec(clip, settings),
  quality: "fast",
});

function useSettledClip(nodeId: string): SettledClip | null {
  const { result: compiled } = useCompiledContext();
  const settings = useWorkflowStore((s) => s.workflow?.settings);
  const clip = compiled?.values[nodeId]?.clip;
  const current =
    isClip(clip) && settings ? { key: stableJson([clip, settings]), clip, settings } : null;
  const latest = useRef(current);
  const [settled, setSettled] = useState(current);
  useLayoutEffect(() => {
    latest.current = current;
  });
  const key = current?.key ?? "";
  useEffect(() => {
    const t = setTimeout(
      () => setSettled((prev) => (prev?.key === key ? prev : latest.current)),
      SETTLE_MS,
    );
    return () => clearTimeout(t);
  }, [key]);
  return settled;
}

function useOnScreen<T extends Element>(): [React.RefObject<T | null>, boolean] {
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

function useThumbSlot(nodeId: string, wanted: boolean): boolean {
  const selected = useUiStore((s) => s.selectedIds.includes(nodeId));
  const claim = useThumbSlots((s) => s.claim);
  const release = useThumbSlots((s) => s.release);
  const holding = useThumbSlots((s) => s.active.includes(nodeId));
  useEffect(() => {
    if (wanted) claim(nodeId, selected);
    else release(nodeId);
  }, [wanted, selected, nodeId, claim, release]);
  useEffect(() => () => release(nodeId), [nodeId, release]);
  return holding;
}

function LiveFrame({ nodeId }: { nodeId: string }) {
  const settings = useWorkflowStore((s) => s.workflow?.settings);
  const zoom = useFlowStore((s) => s.transform[2]);
  const [ref, visible] = useOnScreen<HTMLDivElement>();
  const settled = useSettledClip(nodeId);
  const holding = useThumbSlot(nodeId, visible && zoom >= MIN_ZOOM && !!settled);
  const spec = useMemo(
    () => (settled ? { ...fastSpec(settled), assets: { projectBase: PROJECT_FILE_BASE } } : null),
    [settled],
  );

  if (!settings || !settled) return null;
  const aspectRatio = `${settings.width} / ${settings.height}`;
  const wide = settings.width / settings.height >= 16 / 9;
  return (
    <div
      ref={ref}
      className="bg-muted/40 mx-3 flex aspect-video items-center justify-center overflow-hidden rounded-md"
    >
      {holding && spec ? (
        <Thumbnail
          component={DemoTimeline}
          inputProps={spec}
          compositionWidth={spec.width}
          compositionHeight={spec.height}
          durationInFrames={Math.max(1, spec.durationInFrames)}
          fps={spec.fps}
          frameToDisplay={frameIn(spec.durationInFrames / 2, spec.durationInFrames)}
          style={wide ? { width: "100%", aspectRatio } : { height: "100%", aspectRatio }}
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
  if (def.category === "Browser" || def.category === "Utility") return null;
  if (!("clip" in def.outputs)) return null;
  return <CardStill nodeId={nodeId} />;
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
    void window.demoflow
      .requestStill({
        projectDir,
        nodeId,
        key: settled.key,
        spec,
        frame: frameIn(spec.durationInFrames / 2, spec.durationInFrames),
        priority: selected || !hash,
      })
      .then((r) => {
        if (r.ready) useStillsStore.getState().ready(projectDir, nodeId, r.hash);
      });
  }, [engine, projectDir, settled, hash, wanted, selected, nodeId]);

  if (engine === false) return <LiveFrame nodeId={nodeId} />;
  if (!hash) return null;
  return <FramePicture src={projectRef(`${PROJECT_DIRS.stills}/${hash}.jpg`)} />;
}
