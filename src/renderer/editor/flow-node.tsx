import { NodeResizer, Position, type Node, type NodeProps } from "@xyflow/react";
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
import type { NodeDef, PortSpec } from "@/shared/nodes/define";
import { getNodeDef } from "@/shared/nodes/registry";
import type { NodeRunStatus } from "@/shared/schema/events";
import type { WorkflowNode } from "@/shared/schema/workflow";
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
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import type { FlowNodeData } from "./convert";
import { CategoryBadge } from "./category-badge";
import { CardRows, NodeThumbnail, useCardRows } from "./node-card";
import { PORT_COLORS, portLabel } from "./ports";
import { useCompiledContext } from "./use-compiled";

type DemoflowFlowNode = Node<FlowNodeData>;

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

export const FlowNode = memo(function FlowNode({ id, data }: NodeProps<DemoflowFlowNode>) {
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

function useNodeProblems(id: string, disabled: boolean | undefined) {
  const runError = useRunStore((s) => s.errors[id]);
  const { result, issues } = useCompiledContext();
  const error =
    runError ??
    result?.errors[id] ??
    issues.find((i) => i.nodeId === id && i.level === "error")?.message;
  const warning = disabled
    ? undefined
    : issues.find((i) => i.nodeId === id && i.level === "warning")?.message;
  return { error, warning };
}

function KnownNode({ id, node, def }: { id: string; node: WorkflowNode; def: NodeDef }) {
  const status = useRunStore((s) => s.statuses[id]);
  const active = useUiStore((s) => s.activeIds.includes(id));
  const rows = useCardRows(node, def);
  const { error, warning } = useNodeProblems(id, node.disabled);
  const indicator: NodeStatus =
    status === "running" ? "loading" : status === "error" ? "error" : "initial";
  const succeeded = status === "done" || status === "cached";

  return (
    <NodeStatusIndicator status={indicator} variant="border">
      {succeeded ? (
        <div className="border-success absolute -top-px -left-px h-[calc(100%+2px)] w-[calc(100%+2px)] rounded-[9px] border-2" />
      ) : null}
      <BaseNode
        className={cn("w-64", node.disabled && "opacity-50", active && "ring-primary/70 ring-2")}
      >
        <NodeHeader title={node.label?.trim() || def.title} def={def} status={status} />
        <BaseNodeContent className="gap-1.5 px-0 py-2">
          <NodePorts def={def} />
          <CardRows rows={rows} />
          <NodeThumbnail nodeId={id} def={def} />
          {status === "running" ? <RunProgress id={id} /> : null}
        </BaseNodeContent>
        <NodeFooter error={error} warning={warning} />
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
      style={{ background: PORT_COLORS[port.type], borderColor: PORT_COLORS[port.type] }}
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
      <Progress value={Math.round(progress.value * 100)} />
      {progress.message ? (
        <span className="text-muted-foreground truncate text-[11px]">{progress.message}</span>
      ) : null}
    </div>
  );
}

function NodeFooter({ error, warning }: { error?: string; warning?: string }) {
  if (error)
    return (
      <BaseNodeFooter className="text-destructive items-start text-xs">
        <span className="line-clamp-3">{error}</span>
      </BaseNodeFooter>
    );
  if (warning)
    return (
      <BaseNodeFooter className="text-muted-foreground items-start text-[11px]">
        <span className="line-clamp-2">{warning}</span>
      </BaseNodeFooter>
    );
  return null;
}

export const GroupFlowNode = memo(function GroupFlowNode({
  id,
  data,
  selected,
}: NodeProps<DemoflowFlowNode>) {
  const title = String(data.node.params.title ?? "Group");
  const color = String(data.node.params.color ?? "#a78bfa");
  const setMeta = useWorkflowStore((s) => s.setNodeMeta);
  return (
    <>
      <NodeResizer
        isVisible={selected}
        minWidth={240}
        minHeight={160}
        onResizeEnd={(_e, p) =>
          setMeta(id, { size: { width: p.width, height: p.height }, position: { x: p.x, y: p.y } })
        }
      />
      <div
        className="size-full rounded-xl border-2 border-dashed"
        style={{ borderColor: `${color}88`, background: `${color}0f` }}
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

export const NoteFlowNode = memo(function NoteFlowNode({ data }: NodeProps<DemoflowFlowNode>) {
  return (
    <div className="w-64 rounded-lg border border-amber-300/40 bg-amber-200/10 p-3 text-xs leading-relaxed whitespace-pre-wrap text-amber-100">
      {String(data.node.params.text ?? "")}
    </div>
  );
});
