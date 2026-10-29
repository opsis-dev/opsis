import { findById } from "@opsis/video/common/lists";
import { Trash2 } from "lucide-react";
import { useMemo } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { WorkflowNode } from "@/generated/bindings";
import { clearLogs, useRunStore } from "../store/run";
import { singleSelected, useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { LogView } from "./log-view";
import { RunNodeList } from "./run-node-list";
import { RunOutputs } from "./run-outputs";

const NO_NODES: WorkflowNode[] = [];

function RunLogs({ filter }: { filter: string | null }) {
  const logs = useRunStore((s) => s.logs);
  const visible = useMemo(
    () => (filter ? logs.filter((l) => l.nodeId === filter) : logs),
    [logs, filter],
  );
  return (
    <div className="flex min-h-0 flex-col">
      <div className="flex items-center justify-between border-b px-3 py-1.5 text-xs">
        <span className="text-muted-foreground">
          {filter ? "Logs for the selected node" : "All logs"}
        </span>
        <Button size="xs" variant="ghost" onClick={clearLogs}>
          <Trash2 data-icon="inline-start" />
          Clear
        </Button>
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <LogView lines={visible} showTime className="p-3" />
      </ScrollArea>
    </div>
  );
}

export function RunPanel() {
  const nodes = useWorkflowStore((s) => s.workflow?.nodes ?? NO_NODES);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const filter = useUiStore((s) => singleSelected(s.selectedIds));
  const filterNode = findById(nodes, filter);

  return (
    <div className="grid h-full grid-cols-[minmax(220px,1fr)_2fr_minmax(240px,1.2fr)] grid-rows-[minmax(0,1fr)] divide-x">
      <RunNodeList nodes={nodes} filter={filter} />
      <RunLogs filter={filter} />
      {projectDir ? <RunOutputs filterNode={filterNode} projectDir={projectDir} /> : null}
    </div>
  );
}
