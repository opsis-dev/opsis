import { getNodeDef } from "@opsis/core/nodes/registry";
import { FileSearch, Film } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

export function ReplayButtons({
  nodeId,
  nodeType,
  size,
}: {
  nodeId: string;
  nodeType: string;
  size: "xs" | "sm";
}) {
  const hasResult = useRunStore((s) => !!s.results[nodeId]);
  const hasTrace = useRunStore((s) => !!s.traces[nodeId]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const setReplay = useUiStore((s) => s.setReplay);
  const capture = getNodeDef(nodeType)?.category === "Capture";
  return (
    <>
      {capture && (hasResult || hasTrace) ? (
        <Button size={size} variant="outline" onClick={() => setReplay(nodeId)}>
          <Film data-icon="inline-start" />
          Replay
        </Button>
      ) : null}
      {hasTrace && projectDir ? (
        <Button size={size} variant="outline" onClick={() => setReplay(nodeId, "trace")}>
          <FileSearch data-icon="inline-start" />
          Trace
        </Button>
      ) : null}
    </>
  );
}
