import { CircleX, Folder, MousePointerClick, type LucideIcon } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { errorMessage } from "@/shared/errors";
import { formatDuration } from "@/shared/format";
import { frameAt, type TraceAction, type TraceModel } from "@/shared/trace";
import { Badge } from "@/renderer/components/ui/badge";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/renderer/components/ui/empty";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { isOneOf } from "@/renderer/lib/one-of";
import { cn } from "@/renderer/lib/utils";
import { useUiStore } from "../store/ui";
import { fileUrl } from "./file-url";
import { FrameOverlay } from "./frame-overlay";
import { ListRow } from "./list-row";
import { MONO_TEXT } from "./log-view";
import { CallTab, ConsoleTab, ErrorsTab, NetworkTab, traceClock } from "./trace-tabs";

const MOMENTS = ["before", "after"] as const;
type Moment = (typeof MOMENTS)[number];

const CLICK_SIZE = 1 / 160;

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

function depths(actions: TraceAction[]): Map<string, number> {
  const parents = new Map(actions.map((a) => [a.id, a.parentId]));
  const out = new Map<string, number>();
  const depthOf = (id: string | undefined): number => {
    if (!id || !parents.has(id)) return -1;
    const known = out.get(id);
    if (known !== undefined) return known;
    const depth = depthOf(parents.get(id)) + 1;
    out.set(id, depth);
    return depth;
  };
  for (const a of actions) depthOf(a.id);
  return out;
}

function NoScreenshots() {
  return (
    <Empty className="border">
      <EmptyHeader>
        <EmptyTitle>No screenshots</EmptyTitle>
        <EmptyDescription>This take has no screenshots.</EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}

function Screenshot({
  trace,
  action,
  moment,
}: {
  trace: TraceModel;
  action: TraceAction | undefined;
  moment: Moment;
}) {
  const frame = action
    ? frameAt(trace.frames, moment === "before" ? action.start : action.end)
    : trace.frames.at(-1);
  if (!frame) return <NoScreenshots />;
  const { width, height } = trace.viewport ?? frame;
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
        <FrameOverlay
          width={width}
          height={height}
          box={action?.box}
          point={action?.point}
          pointSize={CLICK_SIZE}
        />
      </div>
    </div>
  );
}

const actionIcon = (a: TraceAction): { icon: LucideIcon; className: string } | null =>
  a.error
    ? { icon: CircleX, className: "text-destructive" }
    : a.group
      ? { icon: Folder, className: "text-muted-foreground" }
      : a.point
        ? { icon: MousePointerClick, className: "text-muted-foreground" }
        : null;

function ActionIcon({ action }: { action: TraceAction }) {
  const found = actionIcon(action);
  if (!found) return <span className="size-4 shrink-0" />;
  const { icon: Icon, className } = found;
  return <Icon className={className} />;
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
  const depth = useMemo(() => depths(trace.actions), [trace]);
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
          <ListRow
            active={a.id === selected}
            style={{ paddingLeft: `${0.5 + (depth.get(a.id) ?? 0)}rem` }}
            onClick={() => {
              onSelect(a.id);
              if (a.nodeId) setSelected([a.nodeId]);
            }}
          >
            <ActionIcon action={a} />
            <span className="flex min-w-0 flex-1 flex-col">
              <span className={cn("truncate", a.group && "font-medium")}>{a.title}</span>
              {a.target ? (
                <span className={cn(MONO_TEXT, "text-muted-foreground truncate")}>{a.target}</span>
              ) : null}
            </span>
            <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
              {formatDuration(a.end - a.start, "precise")}
            </span>
          </ListRow>
        </li>
      ))}
    </ol>
  );
}

function pickAction(trace: TraceModel, id: string | undefined): TraceAction | undefined {
  return (
    trace.actions.find((a) => a.id === id) ??
    trace.actions.find((a) => a.error) ??
    trace.actions.findLast((a) => a.group) ??
    trace.actions.at(-1)
  );
}

function TraceDetails({ trace }: { trace: TraceModel }) {
  const [selectedId, setSelectedId] = useState<string>();
  const [moment, setMoment] = useState<Moment>("after");
  const selected = useMemo(() => pickAction(trace, selectedId), [trace, selectedId]);
  const at = traceClock(trace);
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
            onValueChange={(v) => isOneOf(MOMENTS, v[0]) && setMoment(v[0])}
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
            <CallTab action={selected} />
          </TabsContent>
          <TabsContent value="console">
            <ConsoleTab trace={trace} action={selected} />
          </TabsContent>
          <TabsContent value="network">
            <NetworkTab trace={trace} action={selected} />
          </TabsContent>
          <TabsContent value="errors">
            <ErrorsTab trace={trace} onSelect={setSelectedId} />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

export function TracePanel({ projectDir, traceId }: { projectDir: string; traceId: string }) {
  const { trace, error } = useTrace(projectDir, traceId);
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
  return <TraceDetails trace={trace} />;
}
