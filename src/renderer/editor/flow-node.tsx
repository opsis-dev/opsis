import { NodeResizer, Position, type NodeProps } from "@xyflow/react";
import {
  CheckCheck,
  CircleCheck,
  CircleSlash,
  CircleX,
  Clock,
  LoaderCircle,
  type LucideIcon,
} from "lucide-react";
import { memo } from "react";
import type { z } from "zod";
import {
  BaseNode,
  BaseNodeContent,
  BaseNodeFooter,
  BaseNodeHeader,
  BaseNodeHeaderTitle,
} from "@/renderer/components/base-node";
import { LabeledHandle } from "@/renderer/components/labeled-handle";
import { NodeStatusIndicator, type NodeStatus } from "@/renderer/components/node-status-indicator";
import { Badge } from "@/renderer/components/ui/badge";
import { Progress } from "@/renderer/components/ui/progress";
import { cn } from "@/renderer/lib/utils";
import { wholePercent } from "@/shared/format";
import type { NodeDef, PortSpec } from "@/shared/nodes/define";
import { getNodeDef, nodeName } from "@/shared/nodes/registry";
import { groupParams, noteParams } from "@/shared/nodes/utility";
import type { NodeRunStatus } from "@/shared/schema/events";
import type { WorkflowNode } from "@/shared/schema/workflow";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { CardRows, useCardRows } from "./card-rows";
import { CategoryBadge } from "./category-badge";
import type { CanvasNode } from "./convert";
import { NodeThumbnail } from "./node-still";
import { portLabel, portTypeColor } from "./ports";
import { useNodeProblems } from "./use-node-problems";

type StatusBadge = {
  label: string;
  icon: LucideIcon;
  variant: "secondary" | "outline" | "destructive";
};

const STATUS: Partial<Record<NodeRunStatus, StatusBadge>> = {
  queued: { label: "Waiting", icon: Clock, variant: "outline" },
  running: { label: "Running", icon: LoaderCircle, variant: "outline" },
  done: { label: "Done", icon: CircleCheck, variant: "outline" },
  cached: { label: "Up to date", icon: CheckCheck, variant: "secondary" },
  skipped: { label: "Skipped", icon: CircleSlash, variant: "outline" },
  error: { label: "Failed", icon: CircleX, variant: "destructive" },
};

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

function paramsOf<S extends z.ZodObject>(schema: S, params: unknown): z.infer<S> {
  const parsed = schema.safeParse(params);
  return parsed.success ? parsed.data : schema.parse({});
}

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

function KnownNode({ id, node, def }: { id: string; node: WorkflowNode; def: NodeDef }) {
  const status = useRunStore((s) => s.statuses[id]);
  const active = useUiStore((s) => s.activeIds.includes(id));
  const rows = useCardRows(node, def);
  const { errors, warnings } = useNodeProblems(id);
  const indicator = (status && INDICATOR[status]) ?? "initial";
  const succeeded = status === "done" || status === "cached";

  return (
    <NodeStatusIndicator status={indicator} variant="border">
      {succeeded ? <div className={DONE_BORDER} /> : null}
      <BaseNode
        className={cn("w-64", node.disabled && "opacity-50", active && "ring-primary/70 ring-2")}
      >
        <NodeHeader title={nodeName(node)} def={def} status={status} />
        <BaseNodeContent className="gap-1.5 px-0 py-2">
          <NodePorts def={def} />
          <CardRows rows={rows} />
          <NodeThumbnail nodeId={id} def={def} />
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
}: {
  title: string;
  def: NodeDef;
  status: NodeRunStatus | undefined;
}) {
  const badge = status && status !== "running" ? STATUS[status] : undefined;
  return (
    <BaseNodeHeader className="border-b">
      <CategoryBadge category={def.category} icon={def.icon} size="md" />
      <BaseNodeHeaderTitle className="truncate text-sm">{title}</BaseNodeHeaderTitle>
      {badge ? (
        <Badge
          variant={badge.variant}
          title={status === "skipped" ? "Something before it failed" : undefined}
        >
          <badge.icon data-icon="inline-start" />
          {badge.label}
        </Badge>
      ) : null}
    </BaseNodeHeader>
  );
}

function PortHandles({ ports, side }: { ports: Record<string, PortSpec>; side: "in" | "out" }) {
  return Object.entries(ports).map(([portId, port]) => (
    <LabeledHandle
      key={portId}
      id={portId}
      type={side === "in" ? "target" : "source"}
      position={side === "in" ? Position.Left : Position.Right}
      title={portLabel(port, side)}
      labelClassName={cn("text-xs", side === "in" && port.optional && "text-muted-foreground")}
      style={{ background: portTypeColor(port.type), borderColor: portTypeColor(port.type) }}
    />
  ));
}

function NodePorts({ def }: { def: NodeDef }) {
  if (!Object.keys(def.inputs).length && !Object.keys(def.outputs).length) return null;
  return (
    <div className="grid grid-cols-2 gap-y-1 text-xs">
      <div className="flex flex-col gap-1">
        <PortHandles ports={def.inputs} side="in" />
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
  const { title, color } = paramsOf(groupParams, data.node.params);
  const setMeta = useWorkflowStore((s) => s.setNodeMeta);
  return (
    <>
      <NodeResizer
        isVisible={selected}
        minWidth={GROUP_MIN_WIDTH}
        minHeight={GROUP_MIN_HEIGHT}
        onResizeEnd={(_e, p) =>
          setMeta(id, { size: { width: p.width, height: p.height }, position: { x: p.x, y: p.y } })
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
    <div className="w-64 rounded-lg border border-amber-300/40 bg-amber-200/10 p-3 text-xs leading-relaxed whitespace-pre-wrap text-amber-100">
      {paramsOf(noteParams, data.node.params).text}
    </div>
  );
});
