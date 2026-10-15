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
import { getNodeDef } from "@/shared/nodes/registry";
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
import { NodeIcon } from "./icons";
import { CardRows, NodeThumbnail, useCardRows } from "./NodeCard";
import { CATEGORY_COLORS, PORT_COLORS, portLabel } from "./ports";
import { useCompiledContext } from "./use-compiled";

export type DemoflowFlowNode = Node<FlowNodeData>;

const FALLBACK_DEF = { card: undefined } as unknown as NonNullable<ReturnType<typeof getNodeDef>>;

const STATUS: Partial<
  Record<
    string,
    { label: string; icon: LucideIcon; variant: "secondary" | "outline" | "destructive" }
  >
> = {
  queued: { label: "Waiting", icon: Clock, variant: "outline" },
  running: { label: "Running", icon: LoaderCircle, variant: "outline" },
  done: { label: "Done", icon: CircleCheck, variant: "outline" },
  cached: { label: "Up to date", icon: CheckCheck, variant: "secondary" },
  skipped: { label: "Skipped", icon: CircleSlash, variant: "outline" },
  error: { label: "Failed", icon: CircleX, variant: "destructive" },
};

export const FlowNode = memo(function FlowNode({ id, data }: NodeProps<DemoflowFlowNode>) {
  const node = data.node;
  const def = getNodeDef(node.type);
  const status = useRunStore((s) => s.statuses[id]);
  const progress = useRunStore((s) => s.progress[id]);
  const runError = useRunStore((s) => s.errors[id]);
  const active = useUiStore((s) => s.activeIds.includes(id));
  const { result, issues } = useCompiledContext();
  const error =
    runError ??
    result?.errors[id] ??
    issues.find((i) => i.nodeId === id && i.level === "error")?.message;
  const warning = issues.find((i) => i.nodeId === id && i.level === "warning")?.message;
  const rows = useCardRows(node, def ?? FALLBACK_DEF);

  if (!def) {
    return (
      <BaseNode className="border-destructive w-60 p-3 text-sm">
        Unknown node <code>{node.type}</code>
      </BaseNode>
    );
  }
  const color = CATEGORY_COLORS[def.category] ?? "#71717a";
  const indicator: NodeStatus =
    status === "running"
      ? "loading"
      : status === "done" || status === "cached"
        ? "success"
        : status === "error"
          ? "error"
          : "initial";
  const inputs = Object.entries(def.inputs);
  const outputs = Object.entries(def.outputs);
  const badge = status ? STATUS[status] : undefined;

  return (
    <NodeStatusIndicator status={indicator} variant="border">
      <BaseNode
        className={cn("w-64", node.disabled && "opacity-50", active && "ring-primary/70 ring-2")}
      >
        <BaseNodeHeader className="border-b">
          <span
            className="grid size-6 shrink-0 place-items-center rounded-md text-black"
            style={{ background: color }}
          >
            <NodeIcon name={def.icon} className="size-3.5" />
          </span>
          <BaseNodeHeaderTitle className="truncate text-sm">
            {node.label || def.title}
          </BaseNodeHeaderTitle>
          {badge && status !== "running" ? (
            <Badge
              variant={badge.variant}
              title={status === "skipped" ? "Something before it failed" : undefined}
            >
              <badge.icon data-icon="inline-start" />
              {badge.label}
            </Badge>
          ) : null}
        </BaseNodeHeader>
        <BaseNodeContent className="gap-1.5 px-0 py-2">
          {inputs.length || outputs.length ? (
            <div className="grid grid-cols-2 gap-y-1 text-xs">
              <div className="flex flex-col gap-1">
                {inputs.map(([portId, port]) => (
                  <LabeledHandle
                    key={portId}
                    id={portId}
                    type="target"
                    position={Position.Left}
                    title={portLabel(portId, port, "in")}
                    labelClassName={cn("text-xs", port.optional && "text-muted-foreground")}
                    style={{
                      background: PORT_COLORS[port.type],
                      borderColor: PORT_COLORS[port.type],
                    }}
                  />
                ))}
              </div>
              <div className="flex flex-col items-end gap-1">
                {outputs.map(([portId, port]) => (
                  <LabeledHandle
                    key={portId}
                    id={portId}
                    type="source"
                    position={Position.Right}
                    title={portLabel(portId, port, "out")}
                    labelClassName="text-xs"
                    style={{
                      background: PORT_COLORS[port.type],
                      borderColor: PORT_COLORS[port.type],
                    }}
                  />
                ))}
              </div>
            </div>
          ) : null}
          <CardRows rows={rows} />
          <NodeThumbnail nodeId={id} def={def} />
          {status === "running" && progress ? (
            <div className="flex flex-col gap-1 px-3">
              <Progress value={Math.round(progress.value * 100)} />
              {progress.message ? (
                <span className="text-muted-foreground truncate text-[11px]">
                  {progress.message}
                </span>
              ) : null}
            </div>
          ) : null}
        </BaseNodeContent>
        {error ? (
          <BaseNodeFooter className="text-destructive items-start text-xs">
            <span className="line-clamp-3">{error}</span>
          </BaseNodeFooter>
        ) : warning && !node.disabled ? (
          <BaseNodeFooter className="text-muted-foreground items-start text-[11px]">
            <span className="line-clamp-2">{warning}</span>
          </BaseNodeFooter>
        ) : null}
      </BaseNode>
    </NodeStatusIndicator>
  );
});

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
