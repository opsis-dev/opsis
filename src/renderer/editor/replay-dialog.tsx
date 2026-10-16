import { CircleX, FolderOpen } from "lucide-react";
import { useRef, useState } from "react";
import { nodeName } from "@/shared/nodes/registry";
import type { RecordingValue } from "@/shared/schema/values";
import { Alert, AlertDescription, AlertTitle } from "@/renderer/components/ui/alert";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/renderer/components/ui/empty";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { cn } from "@/renderer/lib/utils";
import { useRunStore } from "../store/run";
import { useUiStore, type ReplayTab } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import { TracePanel } from "./trace-panel";
import { formatDuration } from "@/shared/format";

export function ReplayDialog() {
  const nodeId = useUiStore((s) => s.replayNodeId);
  const setReplay = useUiStore((s) => s.setReplay);
  const tab = useUiStore((s) => s.replayTab);
  const setTab = useUiStore((s) => s.setReplayTab);
  const setSelected = useUiStore((s) => s.setSelected);
  const result = useRunStore((s) => (nodeId ? s.results[nodeId] : undefined));
  const latestTrace = useRunStore((s) => (nodeId ? s.traces[nodeId] : undefined));
  const lastError = useRunStore((s) => (nodeId ? s.errors[nodeId] : undefined));
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const node = useWorkflowStore((s) => s.workflow?.nodes.find((n) => n.id === nodeId));
  const video = useRef<HTMLVideoElement>(null);
  const [time, setTime] = useState(0);

  const recording = result?.outputs.recording as RecordingValue | undefined;
  const image = result?.outputs.image as { src: string | null } | undefined;
  const traceId = latestTrace ?? result?.traceId;
  const actions = recording?.log?.actions ?? [];
  const activeIndex = actions.findLastIndex((a) => a.t <= time * 1000);

  const footage = (
    <div className="grid min-h-0 grid-cols-[1fr_340px] gap-4">
      <div className="flex min-w-0 flex-col gap-3">
        {lastError ? (
          <Alert variant="destructive">
            <CircleX />
            <AlertTitle>The last take failed</AlertTitle>
            <AlertDescription>
              {lastError.split("\n")[0]}
              {recording?.src ? " The footage below is from the last successful take." : ""}
              {traceId ? " Open the Trace tab to see where it stopped." : ""}
            </AlertDescription>
          </Alert>
        ) : null}
        {recording?.src ? (
          <video
            ref={video}
            src={fileUrl(recording.src)}
            controls
            className="w-full rounded-md bg-black"
            onTimeUpdate={(e) => setTime(e.currentTarget.currentTime)}
          />
        ) : image?.src ? (
          <img src={fileUrl(image.src)} alt="Captured still" className="w-full rounded-md" />
        ) : (
          <Empty>
            <EmptyHeader>
              <EmptyTitle>No footage yet</EmptyTitle>
              <EmptyDescription>Run this node to record it.</EmptyDescription>
            </EmptyHeader>
          </Empty>
        )}
        {recording?.src && projectDir ? (
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() =>
                void window.demoflow.revealInFolder(
                  projectDir,
                  recording.src!.replace(/^project:/, ""),
                )
              }
            >
              <FolderOpen data-icon="inline-start" />
              Reveal file
            </Button>
          </div>
        ) : null}
      </div>
      <ScrollArea className="max-h-[74vh]">
        <ol className="flex flex-col gap-1">
          {actions.map((a, i) => (
            <li key={a.id}>
              <Button
                variant="ghost"
                size="sm"
                className={cn(
                  "h-auto w-full justify-start gap-2 py-1.5 text-left",
                  i === activeIndex && "bg-muted",
                )}
                onClick={() => {
                  if (video.current) video.current.currentTime = a.t / 1000;
                  if (a.nodeId) setSelected([a.nodeId]);
                }}
              >
                <span className="text-muted-foreground w-12 shrink-0 text-right tabular-nums">
                  {formatDuration(a.t)}
                </span>
                <Badge variant="outline">{a.kind}</Badge>
                <span className="flex min-w-0 flex-col">
                  <span className="truncate">{a.label ?? a.text ?? ""}</span>
                  {a.selectorUsed ? (
                    <span className="text-muted-foreground truncate font-mono text-[11px]">
                      {a.selectorUsed}
                    </span>
                  ) : null}
                </span>
              </Button>
            </li>
          ))}
        </ol>
      </ScrollArea>
    </div>
  );

  return (
    <Dialog open={!!nodeId} onOpenChange={(open) => !open && setReplay(null)}>
      <DialogContent className="flex h-[94vh] w-[94vw] max-w-[94vw] flex-col gap-4 sm:max-w-[94vw]">
        <DialogHeader>
          <DialogTitle>{node ? nodeName(node) : nodeId}</DialogTitle>
          <DialogDescription>
            {recording
              ? `${formatDuration(recording.durationMs)} · ${actions.length} actions · ${recording.viewport.width}×${recording.viewport.height}`
              : "Last run of this node"}
          </DialogDescription>
        </DialogHeader>
        {traceId && projectDir ? (
          <Tabs
            value={tab}
            onValueChange={(v) => setTab(v as ReplayTab)}
            className="min-h-0 flex-1"
          >
            <TabsList>
              <TabsTrigger value="replay">Replay</TabsTrigger>
              <TabsTrigger value="trace">Trace</TabsTrigger>
            </TabsList>
            <TabsContent value="replay">{footage}</TabsContent>
            <TabsContent value="trace" className="min-h-0">
              <TracePanel projectDir={projectDir} traceId={traceId} />
            </TabsContent>
          </Tabs>
        ) : (
          footage
        )}
      </DialogContent>
    </Dialog>
  );
}
