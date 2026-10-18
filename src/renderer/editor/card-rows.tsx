import { useMemo } from "react";
import { useShallow } from "zustand/react/shallow";
import { isClip } from "@/shared/compile/clip";
import { workflowVars } from "@/shared/compile/compile";
import { errorMessage } from "@/shared/errors";
import { uniqueKeys } from "@/shared/lists";
import type { CardRow } from "@/shared/nodes/card";
import { interpolatedParams, type NodeDef } from "@/shared/nodes/define";
import type { WorkflowNode } from "@/shared/schema/workflow";
import { cn } from "@/renderer/lib/utils";
import { useRunStore } from "../store/run";
import { useSecretsStore } from "../store/secrets";
import { useWorkflowStore } from "../store/workflow";
import { NodeIcon } from "./node-icon";
import { useCompiledContext } from "./use-compiled";
import { openWorkflow } from "./use-open-workflow";

const MAX_ROWS = 4;

const TONE_CLASS: Record<NonNullable<CardRow["tone"]> | "plain", string> = {
  warn: "text-warning",
  muted: "text-muted-foreground",
  plain: "text-foreground/85",
};

function useCardInputs(node: WorkflowNode) {
  const variables = useWorkflowStore((s) => openWorkflow(s).variables);
  const name = useWorkflowStore((s) => openWorkflow(s).name);
  const settings = useWorkflowStore((s) => openWorkflow(s).settings);
  const incoming = useWorkflowStore(
    useShallow((s) => openWorkflow(s).edges.filter((e) => e.target === node.id)),
  );
  const result = useRunStore((s) => s.results[node.id]);
  const secrets = useSecretsStore((s) => s.names);
  const { result: compiled } = useCompiledContext();
  const clip = compiled?.values[node.id]?.clip;
  const clipFrames = isClip(clip) ? clip.durationInFrames : undefined;
  const connected = useMemo(() => incoming.map((e) => e.targetHandle), [incoming]);
  const inputFrames = useMemo(() => {
    const out: Partial<Record<string, number>> = {};
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
    if (!def.card) return [];
    const params = interpolatedParams(def, node.params, workflowVars({ name, variables }));
    try {
      return def
        .card({ params, settings, result, secrets, connected, clipFrames, inputFrames })
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
          <span className="truncate">{r.text}</span>
        </li>
      ))}
    </ul>
  );
}
