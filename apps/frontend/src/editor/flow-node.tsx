import { wholePercent } from "@opsis/video/common/format";
import { NodeResizer, Position, type NodeProps } from "@xyflow/react";
import {
  CheckCheck,
  CircleCheck,
  CircleSlash,
  CircleX,
  Clock,
  type LucideIcon,
  RefreshCw,
} from "lucide-react";
import { memo, useMemo } from "react";
import {
  BaseNode,
  BaseNodeContent,
  BaseNodeFooter,
  BaseNodeHeader,
  BaseNodeHeaderTitle,
} from "@/components/base-node";
import { LabeledHandle } from "@/components/labeled-handle";
import { NodeStatusIndicator, type NodeStatus } from "@/components/node-status-indicator";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { CatalogPort, EditorNode, NodeRunStatus, WorkflowNode } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { getNodeDef, nodeName } from "@/store/catalog";
import { inputsBeforeCompile } from "@/workflow/graph";
import { CARD_WIDTH, groupLook, noteText } from "@/workflow/nodes";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { CardRows, useCardRows } from "./card-rows";
import { NodeBadge } from "./category-badge";
import type { CanvasNode } from "./convert";
import { GLYPH_HOST } from "./node-glyph";
import { portTypeColor } from "./ports";
import { useCompiledContext } from "./use-compiled";
import { useNodeProblems } from "./use-node-problems";

type StatusBadge = {
  label: string;
  icon: LucideIcon;
  variant: "secondary" | "outline" | "destructive";
};

const STATUS: Partial<Record<NodeRunStatus, StatusBadge>> = {
  queued: { label: "Waiting", icon: Clock, variant: "outline" },
  done: { label: "Done", icon: CircleCheck, variant: "outline" },
  cached: { label: "Up to date", icon: CheckCheck, variant: "secondary" },
  skipped: { label: "Skipped", icon: CircleSlash, variant: "outline" },
  error: { label: "Failed", icon: CircleX, variant: "destructive" },
};

const CHANGED: StatusBadge = { label: "Changed", icon: RefreshCw, variant: "outline" };

const SETTLED: ReadonlySet<NodeRunStatus | undefined> = new Set([undefined, "done", "cached"]);

const INDICATOR: Partial<Record<NodeRunStatus, NodeStatus>> = {
  running: "loading",
  error: "error",
};

const DONE_BORDER =
  "border-success absolute -top-px -left-px h-[calc(100%+2px)] w-[calc(100%+2px)] rounded-[9px] border-2";

const GROUP_MIN_WIDTH = 240;
const GROUP_MIN_HEIGHT = 160;
const GROUP_BORDER_MIX = "53%";
const GROUP_FILL_MIX = "6%";

const FOOTERS = {
  error: { className: "text-destructive text-xs", clamp: "line-clamp-3" },
  warning: { className: "text-muted-foreground text-[11px]", clamp: "line-clamp-2" },
};

export const FlowNode = memo(function FlowNode({ id, data }: NodeProps<CanvasNode>) {
  const def = getNodeDef(data.node.type);
  if (!def) {
    return (
      <BaseNode className="border-destructive w-60 p-3 text-sm">
        Unknown node <code>{data.node.type}</code>
      </BaseNode>
    );
  }
  return <KnownNode id={id} node={data.node} def={def} />;
});

function KnownNode({ id, node, def }: { id: string; node: WorkflowNode; def: EditorNode }) {
  const status = useRunStore((s) => s.statuses[id]);
  const active = useUiStore((s) => s.activeIds.includes(id));
  const rows = useCardRows(node);
  const { errors, warnings } = useNodeProblems(id);
  const changed = useCompiledContext().outdated.includes(id) && SETTLED.has(status);
  const indicator = (status && INDICATOR[status]) ?? "initial";
  const succeeded = !changed && (status === "done" || status === "cached");

  return (
    <NodeStatusIndicator status={indicator} variant="border">
      {succeeded ? <div className={DONE_BORDER} /> : null}
      <BaseNode
        {...GLYPH_HOST}
        className={cn(node.disabled && "opacity-50", active && "ring-primary/70 ring-2")}
        style={{ width: CARD_WIDTH }}
      >
        <NodeHeader title={nodeName(node)} def={def} status={status} changed={changed} />
        <BaseNodeContent className="gap-1.5 px-0 py-2">
          <NodePorts id={id} def={def} />
          <CardRows rows={rows} />
          {status === "running" ? <RunProgress id={id} /> : null}
        </BaseNodeContent>
        <NodeFooter error={errors[0]} warning={node.disabled ? undefined : warnings[0]} />
      </BaseNode>
    </NodeStatusIndicator>
  );
}

function NodeHeader({
  title,
  def,
  status,
  changed,
}: {
  title: string;
  def: EditorNode;
  status: NodeRunStatus | undefined;
  changed: boolean;
}) {
  const badge = changed ? CHANGED : status ? STATUS[status] : undefined;
  const tip = changed
    ? "Changed since the last run"
    : status === "skipped"
      ? "Something before it failed"
      : undefined;
  return (
    <BaseNodeHeader className="border-b">
      <NodeBadge def={def} size="md" />
      <BaseNodeHeaderTitle className="truncate text-sm">{title}</BaseNodeHeaderTitle>
      {badge ? (
        <Badge variant={badge.variant} title={tip}>
          <badge.icon data-icon="inline-start" />
          {badge.label}
        </Badge>
      ) : null}
    </BaseNodeHeader>
  );
}

function PortHandles({ ports, side }: { ports: Record<string, CatalogPort>; side: "in" | "out" }) {
  return Object.entries(ports).map(([portId, port]) => {
    const color = portTypeColor(port.type);
    return (
      <LabeledHandle
        key={portId}
        id={portId}
        type={side === "in" ? "target" : "source"}
        position={side === "in" ? Position.Left : Position.Right}
        title={port.label}
        labelClassName={cn("text-xs", side === "in" && port.optional && "text-muted-foreground")}
        style={{ background: color, borderColor: color }}
      />
    );
  });
}

const NO_EDGES: [] = [];

function NodePorts({ id, def }: { id: string; def: EditorNode }) {
  const compiled = useCompiledContext().inputs[id];
  const edges = useWorkflowStore((s) => s.workflow?.edges ?? NO_EDGES);
  const fallback = useMemo(() => inputsBeforeCompile(def, edges, id), [def, edges, id]);
  const inputs = compiled ?? fallback;
  if (!Object.keys(inputs).length && !Object.keys(def.outputs).length) return null;
  return (
    <div className="grid grid-cols-2 gap-y-1 text-xs">
      <div className="flex flex-col gap-1">
        <PortHandles ports={inputs} side="in" />
      </div>
      <div className="flex flex-col items-end gap-1">
        <PortHandles ports={def.outputs} side="out" />
      </div>
    </div>
  );
}

function RunProgress({ id }: { id: string }) {
  const progress = useRunStore((s) => s.progress[id]);
  if (!progress) return null;
  return (
    <div className="flex flex-col gap-1 px-3">
      <Progress value={wholePercent(progress.value)} />
      {progress.message ? (
        <span className="text-muted-foreground truncate text-[11px]">{progress.message}</span>
      ) : null}
    </div>
  );
}

function NodeFooter({ error, warning }: { error?: string; warning?: string }) {
  const message = error ?? warning;
  if (!message) return null;
  const style = FOOTERS[error ? "error" : "warning"];
  return (
    <BaseNodeFooter className={cn("items-start", style.className)}>
      <span className={style.clamp}>{message}</span>
    </BaseNodeFooter>
  );
}

export const GroupFlowNode = memo(function GroupFlowNode({
  id,
  data,
  selected,
}: NodeProps<CanvasNode>) {
  const { title, color } = groupLook(data.node);
  const setMeta = workflowEdits.setNodeMeta;
  return (
    <>
      <NodeResizer
        isVisible={selected}
        minWidth={GROUP_MIN_WIDTH}
        minHeight={GROUP_MIN_HEIGHT}
        onResizeEnd={(_e, p) =>
          setMeta(id, {
            size: { width: Math.round(p.width), height: Math.round(p.height) },
            position: { x: p.x, y: p.y },
          })
        }
      />
      <div
        className="size-full rounded-xl border-2 border-dashed"
        style={{
          borderColor: `color-mix(in srgb, ${color} ${GROUP_BORDER_MIX}, transparent)`,
          background: `color-mix(in srgb, ${color} ${GROUP_FILL_MIX}, transparent)`,
        }}
      >
        <div
          className="rounded-tl-xl rounded-br-xl px-3 py-1 text-xs font-semibold text-black"
          style={{ background: color, width: "fit-content" }}
        >
          {title}
        </div>
      </div>
    </>
  );
});

export const NoteFlowNode = memo(function NoteFlowNode({ data }: NodeProps<CanvasNode>) {
  return (
    <div
      className="rounded-lg border border-amber-300/40 bg-amber-200/10 p-3 text-xs leading-relaxed whitespace-pre-wrap text-amber-100"
      style={{ width: CARD_WIDTH }}
    >
      {noteText(data.node)}
    </div>
  );
});
