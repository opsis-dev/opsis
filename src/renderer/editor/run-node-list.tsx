import { useMemo } from "react";
import { useShallow } from "zustand/react/shallow";
import { wholePercent } from "@/shared/format";
import { nodeName, nodeNameById } from "@/shared/nodes/registry";
import type { WorkflowNode } from "@/shared/schema/workflow";
import { Progress } from "@/renderer/components/ui/progress";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { cn } from "@/renderer/lib/utils";
import { AnimatedPercent } from "../app/motion";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { RunningLabel, StopRunButton } from "./run-controls";
import { StatusIcon, type RowState } from "./status-icon";

function FailureLine({
  message,
  failed,
  skipped,
  nameOf,
}: {
  message: string;
  failed: string[];
  skipped: number;
  nameOf: (id: string) => string;
}) {
  if (failed.length === 0) return <p className="text-destructive px-2 pb-2 text-xs">{message}</p>;
  const select = (id: string) => useUiStore.getState().setSelected([id]);
  const names = failed.map((id, i) => (
    <span key={id}>
      {i > 0 ? ", " : null}
      <button
        type="button"
        className="font-medium underline underline-offset-2"
        onClick={() => select(id)}
      >
        {nameOf(id)}
      </button>
    </span>
  ));
  return (
    <p className="text-destructive px-2 pb-2 text-xs">
      {failed.length === 1 ? (
        <>{names} failed</>
      ) : (
        <>
          {failed.length} nodes failed: {names}
        </>
      )}
      {skipped ? `, ${skipped} skipped` : null}
    </p>
  );
}

function RunHeader({ running, failed }: { running: boolean; failed: boolean }) {
  return (
    <div className="flex items-center justify-between px-2 pb-2">
      {running ? (
        <RunningLabel className="text-sm font-medium" />
      ) : (
        <span className="text-sm font-medium">{failed ? "Last run failed" : "Nodes"}</span>
      )}
      {running ? <StopRunButton size="xs" /> : null}
    </div>
  );
}

function NodeRow({ node, active }: { node: WorkflowNode; active: boolean }) {
  const status = useRunStore((s) => s.statuses[node.id]);
  const failedBefore = useRunStore((s) => !!s.errors[node.id]);
  const progress = useRunStore((s) => s.progress[node.id]?.value);
  const state: RowState = status ?? (failedBefore ? "failedBefore" : "fromLastRun");
  return (
    <button
      type="button"
      onClick={() => useUiStore.getState().setSelected([node.id])}
      className={cn(
        "hover:bg-muted flex flex-col gap-1 rounded-md px-2 py-1.5 text-left",
        active && "bg-muted",
      )}
    >
      <div className="flex items-center gap-2 text-xs">
        <StatusIcon state={state} />
        <span className="flex-1 truncate">{nodeName(node)}</span>
      </div>
      {status === "running" && progress !== undefined ? (
        <div className="flex items-center gap-2 pl-6">
          <Progress className="flex-1" value={wholePercent(progress)} />
          <AnimatedPercent
            className="text-muted-foreground w-10 text-right text-[11px]"
            fraction={progress}
          />
        </div>
      ) : null}
    </button>
  );
}

export function RunNodeList({ nodes, filter }: { nodes: WorkflowNode[]; filter: string | null }) {
  const { running, lastError, failedNodeIds, skippedNodeIds, statuses, errors, results } =
    useRunStore(
      useShallow((s) => ({
        running: s.running,
        lastError: s.lastError,
        failedNodeIds: s.failedNodeIds,
        skippedNodeIds: s.skippedNodeIds,
        statuses: s.statuses,
        errors: s.errors,
        results: s.results,
      })),
    );
  const rows = useMemo(
    () => nodes.filter((n) => statuses[n.id] || errors[n.id] || results[n.id]?.artifacts.length),
    [nodes, statuses, errors, results],
  );
  const nameOf = (id: string) => nodeNameById({ nodes }, id, "A deleted node");

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col p-2">
        <RunHeader running={running} failed={!!lastError} />
        {lastError && !running ? (
          <FailureLine
            message={lastError}
            failed={failedNodeIds}
            skipped={skippedNodeIds.length}
            nameOf={nameOf}
          />
        ) : null}
        {rows.length === 0 ? (
          <p className="text-muted-foreground px-2 text-xs">
            Press Run to see how each node is doing.
          </p>
        ) : null}
        {rows.map((n) => (
          <NodeRow key={n.id} node={n} active={filter === n.id} />
        ))}
      </div>
    </ScrollArea>
  );
}
