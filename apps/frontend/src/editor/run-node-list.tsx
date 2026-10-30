import { wholePercent } from "@opsis/video/common/format";
import { useMemo } from "react";
import { useShallow } from "zustand/react/shallow";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { WorkflowNode } from "@/generated/bindings";
import { CANCELLED } from "@/lib/errors";
import { cn } from "@/lib/utils";
import { nodeName, nodeNameById } from "@/store/catalog";
import { AnimatedPercent } from "../app/motion";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { RunningLabel, StopRunButton } from "./run-controls";
import { StatusIcon, type RowState } from "./status-icon";
import { useCompiledContext } from "./use-compiled";

const FAILURE_LINE = "text-destructive px-2 pb-2 text-xs";

const SETTLED = new Set<RowState>(["done", "cached", "fromLastRun"]);

function FailureLinks({ failed, nameOf }: { failed: string[]; nameOf: (id: string) => string }) {
  function select(id: string) {
    return useUiStore.getState().setSelected([id]);
  }
  return failed.map((id, i) => (
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
}

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
  if (failed.length === 0) return <p className={FAILURE_LINE}>{message}</p>;
  const names = <FailureLinks failed={failed} nameOf={nameOf} />;
  return (
    <p className={FAILURE_LINE}>
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

function RunHeader({
  running,
  failed,
  stopped,
}: {
  running: boolean;
  failed: boolean;
  stopped: boolean;
}) {
  return (
    <div className="flex items-center justify-between px-2 pb-2">
      {running ? (
        <RunningLabel className="text-sm font-medium" />
      ) : (
        <span className="text-sm font-medium">
          {stopped ? "Last run stopped" : failed ? "Last run failed" : "Nodes"}
        </span>
      )}
      {running ? <StopRunButton size="xs" /> : null}
    </div>
  );
}

function NodeRow({ node, active }: { node: WorkflowNode; active: boolean }) {
  const status = useRunStore((s) => s.statuses[node.id]);
  const failedBefore = useRunStore((s) => !!s.errors[node.id]);
  const progress = useRunStore((s) => s.progress[node.id]?.value);
  const changed = useCompiledContext().outdated.includes(node.id);
  const shown: RowState = status ?? (failedBefore ? "failedBefore" : "fromLastRun");
  const state: RowState = changed && SETTLED.has(shown) ? "changed" : shown;
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
  function nameOf(id: string) {
    return nodeNameById(nodes, id, "A deleted node");
  }

  return (
    <ScrollArea className="h-full">
      <div className="flex flex-col p-2">
        <RunHeader running={running} failed={!!lastError} stopped={lastError === CANCELLED} />
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
