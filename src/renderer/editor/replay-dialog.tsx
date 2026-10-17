import { CircleX, FolderOpen } from "lucide-react";
import { useRef, useState } from "react";
import { formatDuration } from "@/shared/format";
import { nodeName } from "@/shared/nodes/registry";
import { outputOf, type RecordingValue } from "@/shared/schema/values";
import type { ActionLog } from "@/shared/schema/actions";
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
import { whenOneOf } from "@/renderer/lib/one-of";
import { cn } from "@/renderer/lib/utils";
import { useRunStore } from "../store/run";
import { REPLAY_TABS, useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { fileUrl } from "./file-url";
import { ListRow } from "./list-row";
import { MONO_TEXT } from "./text-styles";
import { revealProjectFile } from "./reveal";
import { TracePanel } from "./trace-panel";
import { findById } from "@/shared/lists";

type ReplayAction = ActionLog["actions"][number];

function ReplayActionList({
  actions,
  activeIndex,
  onPick,
}: {
  actions: ReplayAction[];
  activeIndex: number;
  onPick: (action: ReplayAction) => void;
}) {
  return (
    <ScrollArea className="max-h-[74vh]">
      <ol className="flex flex-col gap-1">
        {actions.map((a, i) => (
          <li key={a.id}>
            <ListRow active={i === activeIndex} onClick={() => onPick(a)}>
              <span className="text-muted-foreground w-12 shrink-0 text-right tabular-nums">
                {formatDuration(a.t)}
              </span>
              <Badge variant="outline">{a.kind}</Badge>
              <span className="flex min-w-0 flex-col">
                <span className="truncate">{a.label ?? a.text ?? ""}</span>
                {a.selectorUsed ? (
                  <span className={cn(MONO_TEXT, "text-muted-foreground truncate")}>
                    {a.selectorUsed}
                  </span>
                ) : null}
              </span>
            </ListRow>
          </li>
        ))}
      </ol>
    </ScrollArea>
  );
}

function Footage({
  recordingSrc,
  imageSrc,
  onTime,
  video,
}: {
  recordingSrc: string | null;
  imageSrc: string | null;
  onTime: (seconds: number) => void;
  video: React.RefObject<HTMLVideoElement | null>;
}) {
  if (recordingSrc)
    return (
      <video
        ref={video}
        src={fileUrl(recordingSrc)}
        controls
        className="w-full rounded-md bg-black"
        onTimeUpdate={(e) => onTime(e.currentTarget.currentTime)}
      />
    );
  if (imageSrc)
    return <img src={fileUrl(imageSrc)} alt="Captured still" className="w-full rounded-md" />;
  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>No footage yet</EmptyTitle>
        <EmptyDescription>Run this node to record it.</EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}

function ReplayFootage({
  recording,
  imageSrc,
  lastError,
  hasTrace,
  projectDir,
}: {
  recording: RecordingValue | undefined;
  imageSrc: string | null;
  lastError: string | undefined;
  hasTrace: boolean;
  projectDir: string | null;
}) {
  const setSelected = useUiStore((s) => s.setSelected);
  const video = useRef<HTMLVideoElement>(null);
  const [time, setTime] = useState(0);
  const src = recording?.src ?? null;
  const actions = recording?.log?.actions ?? [];
  const activeIndex = actions.findLastIndex((a) => a.t <= time * 1000);

  return (
    <div className="grid min-h-0 grid-cols-[1fr_340px] gap-4">
      <div className="flex min-w-0 flex-col gap-3">
        {lastError ? (
          <Alert variant="destructive">
            <CircleX />
            <AlertTitle>The last take failed</AlertTitle>
            <AlertDescription>
              {lastError.split("\n")[0]}
              {src ? " The footage below is from the last successful take." : ""}
              {hasTrace ? " Open the Trace tab to see where it stopped." : ""}
            </AlertDescription>
          </Alert>
        ) : null}
        <Footage recordingSrc={src} imageSrc={imageSrc} onTime={setTime} video={video} />
        {src && projectDir ? (
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => void revealProjectFile(projectDir, src)}
            >
              <FolderOpen data-icon="inline-start" />
              Reveal file
            </Button>
          </div>
        ) : null}
      </div>
      <ReplayActionList
        actions={actions}
        activeIndex={activeIndex}
        onPick={(a) => {
          if (video.current) video.current.currentTime = a.t / 1000;
          if (a.nodeId) setSelected([a.nodeId]);
        }}
      />
    </div>
  );
}

function ReplayContent({ nodeId }: { nodeId: string }) {
  const tab = useUiStore((s) => s.replayTab);
  const setTab = useUiStore((s) => s.setReplayTab);
  const result = useRunStore((s) => s.results[nodeId]);
  const latestTrace = useRunStore((s) => s.traces[nodeId]);
  const lastError = useRunStore((s) => s.errors[nodeId]);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const node = useWorkflowStore((s) => findById(s.workflow?.nodes, nodeId));
  const recording = outputOf(result, "recording", "recording");
  const imageSrc = outputOf(result, "image", "image")?.src ?? null;
  const traceId = latestTrace ?? result?.traceId;
  const actionCount = recording?.log?.actions.length ?? 0;

  const footage = (
    <ReplayFootage
      recording={recording}
      imageSrc={imageSrc}
      lastError={lastError}
      hasTrace={!!traceId}
      projectDir={projectDir}
    />
  );

  return (
    <>
      <DialogHeader>
        <DialogTitle>{node ? nodeName(node) : nodeId}</DialogTitle>
        <DialogDescription>
          {recording
            ? `${formatDuration(recording.durationMs)} · ${actionCount} actions · ${recording.viewport.width}×${recording.viewport.height}`
            : "Last run of this node"}
        </DialogDescription>
      </DialogHeader>
      {traceId && projectDir ? (
        <Tabs value={tab} onValueChange={whenOneOf(REPLAY_TABS, setTab)} className="min-h-0 flex-1">
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
    </>
  );
}

export function ReplayDialog() {
  const nodeId = useUiStore((s) => s.replayNodeId);
  const setReplay = useUiStore((s) => s.setReplay);
  return (
    <Dialog open={!!nodeId} onOpenChange={(open) => !open && setReplay(null)}>
      <DialogContent className="flex h-[94vh] w-[94vw] max-w-[94vw] flex-col gap-4 sm:max-w-[94vw]">
        {nodeId ? <ReplayContent key={nodeId} nodeId={nodeId} /> : null}
      </DialogContent>
    </Dialog>
  );
}
