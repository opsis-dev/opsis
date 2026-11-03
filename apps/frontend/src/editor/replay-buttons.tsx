import { FileSearch, Film } from "lucide-react";
import { Button } from "@/components/ui/button";
import { commands } from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";
import { outputOf } from "@/workflow/values";
import { useRunStore } from "../store/run";
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
  const hasTake = useRunStore((s) => !!outputOf(s.results[nodeId], "recording", "recording"));
  const hasTrace = useRunStore((s) => !!s.traces[nodeId]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const capture = getNodeDef(nodeType)?.category === "Capture" || hasTake;
  return (
    <>
      {capture && (hasResult || hasTrace) ? (
        <Button
          size={size}
          variant="outline"
          onClick={() => void commands.showReplay(nodeId, "replay")}
        >
          <Film data-icon="inline-start" />
          Replay
        </Button>
      ) : null}
      {hasTrace && projectDir ? (
        <Button
          size={size}
          variant="outline"
          onClick={() => void commands.showReplay(nodeId, "trace")}
        >
          <FileSearch data-icon="inline-start" />
          Trace
        </Button>
      ) : null}
    </>
  );
}
