import { CircleAlert, CircleX, Folder, MousePointerClick } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { frameAt, type TraceAction, type TraceModel } from "@/shared/trace";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/renderer/components/ui/empty";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/renderer/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { cn } from "@/renderer/lib/utils";
import { useUiStore } from "../store/ui";
import { fileUrl } from "./file-url";
import { errorMessage } from "@/shared/errors";
import { formatBytes, formatDuration } from "@/shared/format";

function useTrace(projectDir: string, traceId: string) {
  const [state, setState] = useState<{ trace?: TraceModel; error?: string; id?: string }>({});
  useEffect(() => {
    let live = true;
    window.demoflow.readTrace(projectDir, traceId).then(
      (trace) => live && setState({ trace, id: traceId }),
      (e: unknown) => live && setState({ error: errorMessage(e), id: traceId }),
    );
    return () => {
      live = false;
    };
  }, [projectDir, traceId]);
  return state.id === traceId ? state : {};
}

function depthOf(action: TraceAction, byId: Map<string, TraceAction>): number {
  let depth = 0;
  for (let p = action.parentId; p && byId.has(p); p = byId.get(p)?.parentId) depth += 1;
  return depth;
}

function Screenshot({
  trace,
  action,
  moment,
}: {
  trace: TraceModel;
  action: TraceAction | undefined;
  moment: "before" | "after";
}) {
  const frame = action
    ? frameAt(trace.frames, moment === "before" ? action.start : action.end)
    : trace.frames.at(-1);
  const width = trace.viewport?.width ?? frame?.width ?? 1280;
  const height = trace.viewport?.height ?? frame?.height ?? 720;
  if (!frame)
    return (
      <Empty className="border">
        <EmptyHeader>
          <EmptyTitle>No screenshots</EmptyTitle>
          <EmptyDescription>This take has no screenshots.</EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  return (
    <div className="grid min-h-0 flex-1 place-items-center" style={{ containerType: "size" }}>
      <div
        className="relative overflow-hidden rounded-md bg-black"
        style={{
          aspectRatio: `${width} / ${height}`,
          width: `min(100cqw, calc(100cqh * ${width / height}))`,
        }}
      >
        <img
          src={fileUrl(frame.src)}
          alt={action?.title ?? "Screenshot"}
          className="absolute inset-0 size-full object-contain"
        />
        <svg
          viewBox={`0 0 ${width} ${height}`}
          className="pointer-events-none absolute inset-0 size-full"
        >
          {action?.box ? (
            <rect
              x={action.box.x}
              y={action.box.y}
              width={action.box.width}
              height={action.box.height}
              rx={6}
              className="fill-primary/10 stroke-primary"
              strokeWidth={width / 480}
            />
          ) : null}
          {action?.point ? (
            <circle
              cx={action.point.x}
              cy={action.point.y}
              r={width / 160}
              className="fill-primary stroke-background"
              strokeWidth={width / 960}
            />
          ) : null}
        </svg>
      </div>
    </div>
  );
}

function ActionList({
  trace,
  selected,
  onSelect,
}: {
  trace: TraceModel;
  selected: string | undefined;
  onSelect: (id: string) => void;
}) {
  const setSelected = useUiStore((s) => s.setSelected);
  const byId = useMemo(() => new Map(trace.actions.map((a) => [a.id, a])), [trace]);
  const list = useRef<HTMLOListElement>(null);
  useEffect(() => {
    list.current
      ?.querySelector<HTMLElement>(`[data-action="${CSS.escape(selected ?? "")}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [selected]);
  return (
    <ol ref={list} className="flex flex-col gap-0.5">
      {trace.actions.map((a) => (
        <li key={a.id} data-action={a.id}>
          <Button
            variant="ghost"
            size="sm"
            className={cn(
              "h-auto w-full justify-start gap-2 py-1.5 text-left",
              a.id === selected && "bg-muted",
            )}
            style={{ paddingLeft: `${0.5 + depthOf(a, byId) * 1}rem` }}
            onClick={() => {
              onSelect(a.id);
              if (a.nodeId) setSelected([a.nodeId]);
            }}
          >
            {a.error ? (
              <CircleX className="text-destructive" />
            ) : a.group ? (
              <Folder className="text-muted-foreground" />
            ) : a.point ? (
              <MousePointerClick className="text-muted-foreground" />
            ) : (
              <span className="size-4 shrink-0" />
            )}
            <span className="flex min-w-0 flex-1 flex-col">
              <span className={cn("truncate", a.group && "font-medium")}>{a.title}</span>
              {a.target ? (
                <span className="text-muted-foreground truncate font-mono text-[11px]">
                  {a.target}
                </span>
              ) : null}
            </span>
            <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
              {formatDuration(a.end - a.start, "precise")}
            </span>
          </Button>
        </li>
      ))}
    </ol>
  );
}

function within(time: number, action: TraceAction | undefined) {
  return !!action && time >= action.start && time <= action.end;
}

export function TracePanel({ projectDir, traceId }: { projectDir: string; traceId: string }) {
  const { trace, error } = useTrace(projectDir, traceId);
  const [selectedId, setSelectedId] = useState<string>();
  const [moment, setMoment] = useState<"before" | "after">("after");

  const selected = useMemo(() => {
    if (!trace) return undefined;
    return (
      trace.actions.find((a) => a.id === selectedId) ??
      trace.actions.find((a) => a.error) ??
      trace.actions.findLast((a) => a.group) ??
      trace.actions.at(-1)
    );
  }, [trace, selectedId]);

  if (error)
    return (
      <Empty className="border">
        <EmptyHeader>
          <EmptyTitle>Couldn't open the trace</EmptyTitle>
          <EmptyDescription>{error}</EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  if (!trace)
    return (
      <div className="grid h-full place-items-center">
        <Spinner />
      </div>
    );

  const at = (t: number) => `+${formatDuration(t - trace.start, "precise")}`;
  const warnings = trace.console.filter((c) => c.level === "error" || c.level === "warning").length;

  return (
    <div className="grid h-full min-h-0 grid-cols-[minmax(260px,340px)_1fr] gap-4">
      <ScrollArea className="h-full min-h-0 rounded-md border p-1">
        <ActionList trace={trace} selected={selected?.id} onSelect={setSelectedId} />
      </ScrollArea>
      <div className="flex min-h-0 min-w-0 flex-col gap-3">
        <div className="flex items-center gap-2">
          <span className="min-w-0 flex-1 truncate text-sm font-medium">
            {selected?.title ?? trace.label}
          </span>
          {selected ? (
            <Badge variant="outline" className="tabular-nums">
              {at(selected.start)} · {formatDuration(selected.end - selected.start, "precise")}
            </Badge>
          ) : null}
          <ToggleGroup
            variant="outline"
            size="sm"
            value={[moment]}
            onValueChange={(v) => v[0] && setMoment(v[0] as "before" | "after")}
          >
            <ToggleGroupItem value="before">Before</ToggleGroupItem>
            <ToggleGroupItem value="after">After</ToggleGroupItem>
          </ToggleGroup>
        </div>
        <Screenshot trace={trace} action={selected} moment={moment} />
        <Tabs defaultValue="call" className="shrink-0">
          <TabsList>
            <TabsTrigger value="call">Call</TabsTrigger>
            <TabsTrigger value="console">
              Console
              {warnings ? <Badge variant="secondary">{warnings}</Badge> : null}
            </TabsTrigger>
            <TabsTrigger value="network">
              Network <Badge variant="secondary">{trace.network.length}</Badge>
            </TabsTrigger>
            <TabsTrigger value="errors">
              Errors
              {trace.errors.length ? (
                <Badge variant="destructive">{trace.errors.length}</Badge>
              ) : null}
            </TabsTrigger>
          </TabsList>
          <TabsContent value="call">
            <ScrollArea className="bg-muted/40 h-48 rounded-md">
              <div className="flex flex-col gap-1 p-2 font-mono text-[11px] leading-relaxed">
                {selected?.error ? (
                  <div className="text-destructive whitespace-pre-wrap">{selected.error}</div>
                ) : null}
                {selected?.target ? <div>{selected.target}</div> : null}
                {(selected?.logs ?? []).map((l, i) => (
                  <div key={i} className="text-muted-foreground whitespace-pre-wrap">
                    {l}
                  </div>
                ))}
              </div>
            </ScrollArea>
          </TabsContent>
          <TabsContent value="console">
            <ScrollArea className="h-48 rounded-md border">
              <ol className="flex flex-col font-mono text-[11px]">
                {trace.console.map((c, i) => (
                  <li
                    key={i}
                    className={cn(
                      "flex items-start gap-2 border-b px-2 py-1",
                      within(c.time, selected) && "bg-muted",
                    )}
                  >
                    <span className="text-muted-foreground w-14 shrink-0 tabular-nums">
                      {at(c.time)}
                    </span>
                    <Badge
                      variant={
                        c.level === "error"
                          ? "destructive"
                          : c.level === "warning"
                            ? "secondary"
                            : "outline"
                      }
                    >
                      {c.level}
                    </Badge>
                    <span className="min-w-0 flex-1 break-all whitespace-pre-wrap">{c.text}</span>
                  </li>
                ))}
              </ol>
            </ScrollArea>
          </TabsContent>
          <TabsContent value="network">
            <ScrollArea className="h-48 rounded-md border">
              <Table className="text-xs">
                <TableHeader>
                  <TableRow>
                    <TableHead>Time</TableHead>
                    <TableHead>Method</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>URL</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead className="text-right">Size</TableHead>
                    <TableHead className="text-right">Duration</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {trace.network.map((r, i) => (
                    <TableRow
                      key={i}
                      data-state={within(r.start, selected) ? "selected" : undefined}
                    >
                      <TableCell className="tabular-nums">{at(r.start)}</TableCell>
                      <TableCell>{r.method}</TableCell>
                      <TableCell>
                        {r.failure ? (
                          <Badge variant="destructive">{r.failure}</Badge>
                        ) : (
                          <Badge variant={r.status >= 400 ? "destructive" : "outline"}>
                            {r.status}
                          </Badge>
                        )}
                      </TableCell>
                      <TableCell className="max-w-96 truncate font-mono" title={r.url}>
                        {r.url}
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {r.resourceType || r.mimeType.split(";")[0]}
                      </TableCell>
                      <TableCell className="text-right tabular-nums">
                        {formatBytes(r.size)}
                      </TableCell>
                      <TableCell className="text-right tabular-nums">
                        {formatDuration(r.duration, "precise")}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </ScrollArea>
          </TabsContent>
          <TabsContent value="errors">
            <ScrollArea className="h-48 rounded-md border">
              {trace.errors.length === 0 ? (
                <p className="text-muted-foreground p-2 text-xs">No errors.</p>
              ) : (
                <ol className="flex flex-col font-mono text-[11px]">
                  {trace.errors.map((e, i) => (
                    <li key={i} className="flex items-start gap-2 border-b px-2 py-1">
                      <CircleAlert className="text-destructive size-3.5 shrink-0" />
                      <span className="text-muted-foreground w-14 shrink-0 tabular-nums">
                        {at(e.time)}
                      </span>
                      <button
                        type="button"
                        className="min-w-0 flex-1 text-left break-all whitespace-pre-wrap"
                        onClick={() => e.actionId && setSelectedId(e.actionId)}
                      >
                        {e.stack ?? e.message}
                      </button>
                    </li>
                  ))}
                </ol>
              )}
            </ScrollArea>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
