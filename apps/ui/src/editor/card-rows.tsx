import { inputTake, isClip, recordingTake, sceneItemCount } from "@opsis/core/compile/clip";
import { errorMessage } from "@opsis/core/errors";
import { uniqueKeys } from "@opsis/core/lists";
import { namedProjectFiles } from "@opsis/core/named-files";
import type { CardContext, CardRow, InputTake } from "@opsis/core/nodes/card";
import { cardParams, cardRows, type NodeDef } from "@opsis/core/nodes/define";
import type { StyleEntry } from "@opsis/core/schema/values";
import type { WorkflowNode } from "@opsis/core/schema/workflow";
import { workflowVars } from "@opsis/core/vars";
import { useEffect, useMemo } from "react";
import { useShallow } from "zustand/react/shallow";
import { cn } from "@/lib/utils";
import { useProjectFiles } from "../store/project-files";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useWorkflowStore } from "../store/workflow";
import { NodeIcon } from "./node-icon";
import { requireWorkflow } from "./require-workflow";
import { useCompiledContext } from "./use-compiled";

const TONE_CLASS: Record<NonNullable<CardRow["tone"]> | "plain", string> = {
  warn: "text-warning",
  muted: "text-muted-foreground",
  plain: "text-foreground/85",
};

function useCardInputs(node: WorkflowNode) {
  const variables = useWorkflowStore((s) => requireWorkflow(s).variables);
  const name = useWorkflowStore((s) => requireWorkflow(s).name);
  const settings = useWorkflowStore((s) => requireWorkflow(s).settings);
  const incoming = useWorkflowStore(
    useShallow((s) => requireWorkflow(s).edges.filter((e) => e.target === node.id)),
  );
  const result = useRunStore((s) => s.results[node.id]);
  const secrets = useSecretsStore((s) => s.names);
  const { result: compiled } = useCompiledContext();
  const clip = compiled?.values[node.id]?.clip;
  const clipFrames = isClip(clip) ? clip.durationInFrames : undefined;
  const connected = useMemo(() => incoming.map((e) => e.targetHandle), [incoming]);
  const inputs = useMemo(() => {
    const frames: Partial<Record<string, number>> = {};
    const items: Partial<Record<string, number>> = {};
    const takes: Partial<Record<string, InputTake>> = {};
    const media: NonNullable<CardContext<unknown>["inputMedia"]> = {};
    const captions: Partial<Record<string, number>> = {};
    const steps: Partial<Record<string, string[]>> = {};
    const style: Partial<Record<string, string[]>> = {};
    const grade: Partial<Record<string, StyleEntry[]>> = {};
    for (const e of incoming) {
      const v = compiled?.values[e.source]?.[e.sourceHandle];
      if (v?.type === "media") media[e.targetHandle] = { kind: v.kind, hasAudio: v.hasAudio };
      if (v?.type === "captions") captions[e.targetHandle] = v.captions.at(-1)?.endMs ?? 0;
      if (v?.type === "browser-session") steps[e.targetHandle] = v.steps.map((s) => s.type);
      if (v?.type === "recording-style") style[e.targetHandle] = v.entries.map((s) => s.type);
      if (v?.type === "grade") grade[e.targetHandle] = v.entries;
      if (v?.type === "recording") {
        const take = recordingTake(v);
        if (take !== undefined) takes[e.targetHandle] = take;
      }
      if (!isClip(v)) continue;
      frames[e.targetHandle] = v.durationInFrames;
      items[e.targetHandle] = sceneItemCount(v);
      const take = inputTake(v);
      if (take !== undefined) takes[e.targetHandle] = take;
    }
    return {
      inputFrames: frames,
      inputItems: items,
      inputTakes: takes,
      inputMedia: media,
      inputCaptionsMs: captions,
      inputSteps: steps,
      inputStyle: style,
      inputGrade: grade,
    };
  }, [incoming, compiled]);
  return useMemo(
    () => ({
      vars: workflowVars({ name, variables }),
      context: {
        settings,
        result,
        secrets,
        connected,
        clipFrames,
        ...inputs,
      },
    }),
    [name, variables, settings, result, secrets, connected, clipFrames, inputs],
  );
}

const NO_FILES: string[] = [];

function useMissingFiles(params: unknown): string[] {
  const refs = useMemo(() => namedProjectFiles(params), [params]);
  const check = useProjectFiles((s) => s.check);
  useEffect(() => {
    if (refs.length) void check(refs);
  }, [refs, check]);
  const missing = useProjectFiles(useShallow((s) => refs.filter((ref) => s.gone[ref])));
  return missing.length ? missing : NO_FILES;
}

const summaryFailed = (error: unknown): CardRow[] => [
  {
    icon: "triangle-alert",
    text: "Couldn't show this node's summary",
    title: errorMessage(error),
    tone: "warn",
  },
];

export function useCardRows(node: WorkflowNode, def: NodeDef): CardRow[] {
  const { vars, context } = useCardInputs(node);
  const shown = useMemo(() => {
    try {
      return { params: cardParams(def, node.params, vars) };
    } catch (error) {
      return { error };
    }
  }, [def, node.params, vars]);
  const missingFiles = useMissingFiles(shown.params);
  return useMemo(() => {
    if (!def.card) return [];
    if (!shown.params) return summaryFailed(shown.error);
    try {
      return cardRows(def, { ...context, missingFiles, params: shown.params });
    } catch (error) {
      return summaryFailed(error);
    }
  }, [def, shown, missingFiles, context]);
}

const MAX_SWATCHES = 8;

export function CardRows({ rows }: { rows: CardRow[] }) {
  if (!rows.length) return null;
  const keys = uniqueKeys(rows.map((r) => `${r.icon}:${r.text}`));
  return (
    <ul className="flex flex-col gap-1 px-3">
      {rows.map((r, i) => (
        <li
          key={keys[i]}
          title={r.title ?? r.text}
          className={cn("flex min-w-0 items-center gap-1.5 text-xs", TONE_CLASS[r.tone ?? "plain"])}
        >
          <NodeIcon
            name={r.icon}
            className={cn("size-3.5 shrink-0", r.tone ? undefined : "text-muted-foreground")}
          />
          {r.swatches?.length ? (
            <span className="flex shrink-0 -space-x-1" aria-hidden>
              {r.swatches.slice(0, MAX_SWATCHES).map((color, j) => (
                <span
                  key={j}
                  className="ring-card size-3 rounded-full ring-1"
                  style={{ background: color }}
                />
              ))}
            </span>
          ) : null}
          <span className="truncate">{r.text}</span>
        </li>
      ))}
    </ul>
  );
}
