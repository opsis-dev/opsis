import {
  Circle,
  CircleCheck,
  CircleSlash,
  CircleX,
  FileSearch,
  Film,
  FolderOpen,
  History,
  Square,
  Trash2,
  type LucideIcon,
} from "lucide-react";
import { useMemo } from "react";
import { getNodeDef } from "@/shared/nodes/registry";
import { exportedArtifacts } from "@/shared/run-progress";
import type { NodeRunStatus } from "@/shared/schema/events";
import type { Artifact } from "@/shared/schema/values";
import { Button } from "@/renderer/components/ui/button";
import { Progress } from "@/renderer/components/ui/progress";
import { TextShimmer } from "@/renderer/components/ui/text-shimmer";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { AnimatedPercent } from "../app/motion";
import { cancelRun } from "./run";

const fileUrl = (rel: string) =>
  `demoflow-file://project/${rel.split("/").map(encodeURIComponent).join("/")}`;

function ArtifactPreview({ artifact }: { artifact: Artifact }) {
  if (artifact.kind === "video")
    return (
      <video
        src={fileUrl(artifact.path)}
        controls
        muted
        className="aspect-video w-full rounded-md bg-black"
      />
    );
  if (artifact.kind === "image")
    return (
      <img
        src={fileUrl(artifact.path)}
        alt={artifact.label ?? artifact.path}
        className="w-full rounded-md"
      />
    );
  if (artifact.kind === "audio")
    return <audio src={fileUrl(artifact.path)} controls className="w-full" />;
  return null;
}

type RowState = NodeRunStatus | "failedBefore" | "fromLastRun";

const STATUS_ICONS: Record<RowState, { icon: LucideIcon; word: string; className: string }> = {
  idle: { icon: Circle, word: "Not run", className: "text-muted-foreground" },
  queued: { icon: Circle, word: "Waiting", className: "text-muted-foreground" },
  running: { icon: Circle, word: "Running", className: "" },
  done: { icon: CircleCheck, word: "Done", className: "text-success" },
  cached: { icon: CircleCheck, word: "Up to date", className: "text-muted-foreground" },
  skipped: { icon: CircleSlash, word: "Skipped", className: "text-muted-foreground" },
  error: { icon: CircleX, word: "Failed", className: "text-destructive" },
  failedBefore: { icon: CircleX, word: "Failed last time", className: "text-destructive/60" },
  fromLastRun: { icon: History, word: "From last run", className: "text-muted-foreground" },
};

function StatusIcon({ state }: { state: RowState }) {
  const { icon: Icon, word, className } = STATUS_ICONS[state];
  return (
    <Tooltip>
      <TooltipTrigger
        render={<span className="flex size-4 shrink-0 items-center justify-center" />}
      >
        {state === "running" ? (
          <Spinner aria-label={word} />
        ) : (
          <Icon aria-label={word} className={cn("size-4", className)} />
        )}
      </TooltipTrigger>
      <TooltipContent>{word}</TooltipContent>
    </Tooltip>
  );
}

function FailureLine({
  message,
  failed,
  skipped,
  nameOf,
  onSelect,
}: {
  message: string;
  failed: string[];
  skipped: number;
  nameOf: (id: string) => string;
  onSelect: (id: string) => void;
}) {
  if (failed.length === 0) return <p className="text-destructive px-2 pb-2 text-xs">{message}</p>;
  const names = failed.map((id, i) => (
    <span key={id}>
      {i > 0 ? ", " : null}
      <button
        type="button"
        className="font-medium underline underline-offset-2"
        onClick={() => onSelect(id)}
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

export function RunPanel() {
  const workflow = useWorkflowStore((s) => s.workflow);
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const {
    running,
    statuses,
    progress,
    logs,
    results,
    traces,
    errors,
    lastError,
    failedNodeIds,
    skippedNodeIds,
    clearLogs,
  } = useRunStore();
  const selected = useUiStore((s) => s.selectedIds);
  const setSelected = useUiStore((s) => s.setSelected);
  const setReplay = useUiStore((s) => s.setReplay);
  const filter = selected.length === 1 ? selected[0] : null;

  const rows = useMemo(
    () =>
      workflow
        ? workflow.nodes.filter(
            (n) => statuses[n.id] || errors[n.id] || results[n.id]?.artifacts.length,
          )
        : [],
    [workflow, statuses, errors, results],
  );
  const exported = useMemo(
    () => (workflow ? exportedArtifacts(workflow.nodes, results) : []),
    [workflow, results],
  );
  const nameOf = (id: string) => {
    const node = workflow?.nodes.find((n) => n.id === id);
    return node ? node.label || getNodeDef(node.type)?.title || node.type : "A deleted node";
  };
  const visibleLogs = filter ? logs.filter((l) => l.nodeId === filter) : logs;
  const artifacts = filter ? (results[filter]?.artifacts ?? []) : exported;

  return (
    <div className="grid h-full grid-cols-[minmax(220px,1fr)_2fr_minmax(240px,1.2fr)] grid-rows-[minmax(0,1fr)] divide-x">
      <ScrollArea className="h-full">
        <div className="flex flex-col p-2">
          <div className="flex items-center justify-between px-2 pb-2">
            {running ? (
              <TextShimmer as="span" className="text-sm font-medium" duration={1.6}>
                Running…
              </TextShimmer>
            ) : (
              <span className="text-sm font-medium">{lastError ? "Last run failed" : "Nodes"}</span>
            )}
            {running ? (
              <Button size="xs" variant="outline" onClick={() => void cancelRun()}>
                <Square data-icon="inline-start" />
                Cancel
              </Button>
            ) : null}
          </div>
          {lastError && !running ? (
            <FailureLine
              message={lastError}
              failed={failedNodeIds}
              skipped={skippedNodeIds.length}
              nameOf={nameOf}
              onSelect={(id) => setSelected([id])}
            />
          ) : null}
          {rows.length === 0 ? (
            <p className="text-muted-foreground px-2 text-xs">
              Press Run to see how each node is doing.
            </p>
          ) : null}
          {rows.map((n) => {
            const status = statuses[n.id];
            const state: RowState = status ? status : errors[n.id] ? "failedBefore" : "fromLastRun";
            return (
              <button
                key={n.id}
                type="button"
                onClick={() => setSelected([n.id])}
                className={cn(
                  "hover:bg-muted flex flex-col gap-1 rounded-md px-2 py-1.5 text-left",
                  filter === n.id && "bg-muted",
                )}
              >
                <div className="flex items-center gap-2 text-xs">
                  <StatusIcon state={state} />
                  <span className="flex-1 truncate">
                    {n.label || getNodeDef(n.type)?.title || n.type}
                  </span>
                </div>
                {status === "running" && progress[n.id] ? (
                  <div className="flex items-center gap-2 pl-6">
                    <Progress className="flex-1" value={Math.round(progress[n.id].value * 100)} />
                    <AnimatedPercent
                      className="text-muted-foreground w-10 text-right text-[11px]"
                      value={progress[n.id].value * 100}
                    />
                  </div>
                ) : null}
              </button>
            );
          })}
        </div>
      </ScrollArea>
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
          <pre className="p-3 font-mono text-[11px] leading-relaxed whitespace-pre-wrap">
            {visibleLogs.map((l, i) => (
              <div
                key={i}
                className={cn(
                  l.level === "error" && "text-destructive",
                  l.level === "warn" && "text-muted-foreground",
                )}
              >
                <span className="text-muted-foreground">
                  {new Date(l.at).toLocaleTimeString()}{" "}
                </span>
                {l.message}
              </div>
            ))}
          </pre>
        </ScrollArea>
      </div>
      <ScrollArea className="h-full">
        <div className="flex flex-col gap-3 p-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">{filter ? "Artifacts" : "Outputs"}</span>
            {projectDir ? (
              <Button
                size="xs"
                variant="outline"
                onClick={() => void window.demoflow.revealInFolder(projectDir, ".demoflow/out")}
              >
                <FolderOpen data-icon="inline-start" />
                Output folder
              </Button>
            ) : null}
          </div>
          {filter && (results[filter] || traces[filter]) ? (
            <div className="flex flex-wrap gap-2">
              {getNodeDef(workflow?.nodes.find((n) => n.id === filter)?.type ?? "")?.category ===
              "Capture" ? (
                <Button size="xs" variant="outline" onClick={() => setReplay(filter)}>
                  <Film data-icon="inline-start" />
                  Replay
                </Button>
              ) : null}
              {traces[filter] && projectDir ? (
                <Button size="xs" variant="outline" onClick={() => setReplay(filter, "trace")}>
                  <FileSearch data-icon="inline-start" />
                  Trace
                </Button>
              ) : null}
            </div>
          ) : null}
          {artifacts.length === 0 ? (
            <p className="text-muted-foreground text-xs">Nothing yet.</p>
          ) : null}
          {artifacts.map((a) => (
            <div key={a.path} className="flex flex-col gap-1.5">
              <ArtifactPreview artifact={a} />
              <Button
                variant="link"
                size="xs"
                className="h-auto justify-start p-0"
                onClick={() =>
                  projectDir && void window.demoflow.revealInFolder(projectDir, a.path)
                }
              >
                {a.label ?? a.path}
              </Button>
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
