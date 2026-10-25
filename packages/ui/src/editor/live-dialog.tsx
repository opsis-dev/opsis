import { useMemo } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useLiveStore, type LiveCapture } from "../store/live";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { ListRow } from "./list-row";
import { LiveView } from "./live-view";
import { LogView } from "./log-view";
import { STATUS_LOOKS, StatusIcon } from "./status-icon";

function StepList({ capture }: { capture: LiveCapture }) {
  const setSelected = useUiStore((s) => s.setSelected);
  return (
    <ol className="flex flex-col gap-1">
      {capture.steps.map((step, i) => (
        <li key={step.nodeId}>
          <ListRow onClick={() => setSelected([step.nodeId])}>
            <span className="text-muted-foreground w-5 shrink-0 text-right tabular-nums">
              {i + 1}
            </span>
            <StatusIcon state={step.status ?? "queued"} />
            <span className="flex flex-col">
              <span className="truncate">{step.label}</span>
              {step.message ? (
                <span className="text-destructive text-xs whitespace-normal">{step.message}</span>
              ) : null}
            </span>
          </ListRow>
        </li>
      ))}
    </ol>
  );
}

export function LiveDialog({ capture }: { capture: LiveCapture }) {
  const open = useLiveStore((s) => s.dialogOpen);
  const setOpen = useLiveStore((s) => s.setDialogOpen);
  const logs = useRunStore((s) => s.logs);
  const nodeLogs = useMemo(
    () => logs.filter((l) => l.nodeId === capture.nodeId),
    [logs, capture.nodeId],
  );
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="grid max-h-[92vh] w-[92vw] max-w-[92vw] grid-cols-[1fr_320px] gap-4 sm:max-w-[92vw]">
        <div className="flex min-w-0 flex-col gap-3">
          <DialogHeader>
            <DialogTitle>{capture.title}</DialogTitle>
            <DialogDescription>
              {capture.viewport.width}×{capture.viewport.height} ·{" "}
              {STATUS_LOOKS[capture.status].word}
              {capture.message ? ` · ${capture.message}` : ""}
            </DialogDescription>
          </DialogHeader>
          <LiveView capture={capture} className="w-full" />
        </div>
        <div className="flex min-h-0 flex-col gap-3">
          <ScrollArea className="max-h-[45vh]">
            <StepList capture={capture} />
          </ScrollArea>
          <ScrollArea className="bg-muted/40 max-h-[30vh] rounded-md">
            <LogView lines={nodeLogs} className="p-2" />
          </ScrollArea>
        </div>
      </DialogContent>
    </Dialog>
  );
}
