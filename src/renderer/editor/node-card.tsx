import { Thumbnail } from "@remotion/player";
import { useStore as useFlowStore } from "@xyflow/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { create } from "zustand";
import { useShallow } from "zustand/react/shallow";
import { clipToSpec, isClip } from "@/shared/compile/clip";
import { workflowVars } from "@/shared/compile/compile";
import type { CardRow } from "@/shared/nodes/card";
import type { NodeDef } from "@/shared/nodes/define";
import { defaultParams } from "@/shared/nodes/registry";
import type { NodeResult } from "@/shared/schema/values";
import type { WorkflowNode } from "@/shared/schema/workflow";
import { interpolate, stableStringify } from "@/shared/vars";
import { DemoTimeline } from "@/video/demo-timeline";
import { cn } from "@/renderer/lib/utils";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import { NodeIcon } from "./icons";
import { useCompiledContext } from "./use-compiled";

const MAX_LIVE_THUMBS = 6;
const MIN_ZOOM = 0.55;

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

export function useCardRows(node: WorkflowNode, def: NodeDef): CardRow[] {
  const variables = useWorkflowStore((s) => s.workflow?.variables);
  const name = useWorkflowStore((s) => s.workflow?.name ?? "");
  const settings = useWorkflowStore((s) => s.workflow?.settings);
  const connected = useWorkflowStore(
    useShallow((s) =>
      (s.workflow?.edges ?? []).filter((e) => e.target === node.id).map((e) => e.targetHandle),
    ),
  );
  const result = useRunStore((s) => s.results[node.id]);
  const secrets = useSecretsStore((s) => s.names);
  const { result: compiled } = useCompiledContext();
  const clip = compiled?.values[node.id]?.clip;
  const clipFrames = isClip(clip) ? clip.durationInFrames : undefined;

  return useMemo(() => {
    if (!def.card || !settings) return [];
    const vars = workflowVars({ name, variables: variables ?? {} } as Parameters<
      typeof workflowVars
    >[0]);
    const raw = node.params ?? {};
    const params = { ...defaultParams(node.type), ...interpolate(raw, vars) };
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
        })
        .filter((r): r is CardRow => !!r && typeof r === "object")
        .slice(0, 4);
    } catch {
      return [];
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

function useOnScreen<T extends Element>(): [React.RefObject<T | null>, boolean] {
  const ref = useRef<T>(null);
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(([entry]) => setVisible(entry.isIntersecting), {
      rootMargin: "120px",
    });
    io.observe(el);
    return () => io.disconnect();
  }, []);
  return [ref, visible];
}

function LiveFrame({ nodeId }: { nodeId: string }) {
  const { result: compiled } = useCompiledContext();
  const settings = useWorkflowStore((s) => s.workflow?.settings);
  const selected = useUiStore((s) => s.selectedIds.includes(nodeId));
  const zoom = useFlowStore((s) => s.transform[2]);
  const claim = useThumbSlots((s) => s.claim);
  const release = useThumbSlots((s) => s.release);
  const holding = useThumbSlots((s) => s.active.includes(nodeId));
  const [ref, visible] = useOnScreen<HTMLDivElement>();
  const clip = compiled?.values[nodeId]?.clip;
  const key = isClip(clip) && settings ? stableStringify([clip, settings]) : "";
  const [settled, setSettled] = useState(key);
  useEffect(() => {
    const t = setTimeout(() => setSettled(key), 400);
    return () => clearTimeout(t);
  }, [key]);

  const wanted = visible && zoom >= MIN_ZOOM && !!settled;
  useEffect(() => {
    if (wanted) claim(nodeId, selected);
    else release(nodeId);
  }, [wanted, selected, nodeId, claim, release]);
  useEffect(() => () => release(nodeId), [nodeId, release]);

  const spec = useMemo(() => {
    if (!settled) return null;
    const [c, s] = JSON.parse(settled) as [
      Parameters<typeof clipToSpec>[0],
      Parameters<typeof clipToSpec>[1],
    ];
    return {
      ...clipToSpec(c, s),
      assets: { projectBase: "demoflow-file://project/" },
      quality: "fast" as const,
    };
  }, [settled]);

  if (!settings || !isClip(clip)) return null;
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
          frameToDisplay={Math.min(
            spec.durationInFrames - 1,
            Math.round(spec.durationInFrames / 2),
          )}
          style={
            settings.width / settings.height >= 16 / 9
              ? { width: "100%", aspectRatio: `${settings.width} / ${settings.height}` }
              : { height: "100%", aspectRatio: `${settings.width} / ${settings.height}` }
          }
          errorFallback={() => null}
        />
      ) : null}
    </div>
  );
}

export function NodeThumbnail({ nodeId, def }: { nodeId: string; def: NodeDef }) {
  const result = useRunStore((s) => s.results[nodeId]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const poster = posterOf(result);
  if (poster && projectDir) {
    return (
      <div className="mx-3 overflow-hidden rounded-md bg-black">
        <img
          src={fileUrl(poster)}
          alt=""
          loading="lazy"
          draggable={false}
          className="block aspect-video w-full object-contain"
        />
      </div>
    );
  }
  if (def.category === "Browser" || def.category === "Utility") return null;
  if (!("clip" in def.outputs)) return null;
  return <LiveFrame nodeId={nodeId} />;
}
