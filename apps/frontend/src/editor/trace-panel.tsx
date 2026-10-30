import { formatPreciseDuration } from "@opsis/video/common/format";
import { findById } from "@opsis/video/common/lists";
import { CircleX, Folder, MousePointerClick } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/components/ui/empty";
import { ScrollArea } from "@/components/ui/scroll-area";
import { frameAt } from "@/editor/trace-frames";
import { commands, type TraceAction, type TraceModel } from "@/generated/bindings";
import { errorMessage } from "@/lib/errors";
import { cn } from "@/lib/utils";
import { LoadingPanel } from "../app/empty-panel";
import { OptionToggle } from "../app/option-toggle";
import { useUiStore } from "../store/ui";
import { fileUrl } from "./file-url";
import { FrameOverlay, OVERLAY_WIDTHS } from "./frame-overlay";
import { ListRow } from "./list-row";
import { MONO_TEXT } from "./text-styles";
import { traceClock, TraceTabs } from "./trace-tabs";

type Moment = "before" | "after";
const MOMENTS: ReadonlyArray<readonly [Moment, string]> = [
  ["before", "Before"],
  ["after", "After"],
];

function useTrace(projectDir: string, traceId: string) {
  const [state, setState] = useState<{ trace?: TraceModel; error?: string; id?: string }>({});
  useEffect(() => {
    let live = true;
    commands.readTrace(projectDir, traceId).then(
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
  function depthOf(id: string | undefined): number {
    if (!id || !parents.has(id)) return -1;
    const known = out.get(id);
    if (known !== undefined) return known;
    const depth = depthOf(parents.get(id)) + 1;
    out.set(id, depth);
    return depth;
  }
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
          pointSize={OVERLAY_WIDTHS.tracePoint}
        />
      </div>
    </div>
  );
}

function ActionIcon({ action }: { action: TraceAction }) {
  if (action.error) return <CircleX className="text-destructive" />;
  if (action.group) return <Folder className="text-muted-foreground" />;
  if (action.point) return <MousePointerClick className="text-muted-foreground" />;
  return <span className="size-4 shrink-0" />;
}

function TraceActionList({
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
              {formatPreciseDuration(a.end - a.start)}
            </span>
          </ListRow>
        </li>
      ))}
    </ol>
  );
}

function pickAction(trace: TraceModel, id: string | undefined): TraceAction | undefined {
  return (
    findById(trace.actions, id) ??
    trace.actions.find((a) => a.error) ??
    trace.actions.findLast((a) => a.group) ??
    trace.actions.at(-1)
  );
}

function ActionHeader({
  trace,
  action,
  moment,
  onMoment,
}: {
  trace: TraceModel;
  action: TraceAction | undefined;
  moment: Moment;
  onMoment: (moment: Moment) => void;
}) {
  const at = traceClock(trace);
  return (
    <div className="flex items-center gap-2">
      <span className="min-w-0 flex-1 truncate text-sm font-medium">
        {action?.title ?? trace.label}
      </span>
      {action ? (
        <Badge variant="outline" className="tabular-nums">
          {at(action.start)} · {formatPreciseDuration(action.end - action.start)}
        </Badge>
      ) : null}
      <OptionToggle choices={MOMENTS} value={moment} onChange={onMoment} />
    </div>
  );
}

function TraceDetails({ trace }: { trace: TraceModel }) {
  const [selectedId, setSelectedId] = useState<string>();
  const [moment, setMoment] = useState<Moment>("after");
  const selected = useMemo(() => pickAction(trace, selectedId), [trace, selectedId]);

  return (
    <div className="grid h-full min-h-0 grid-cols-[minmax(260px,340px)_1fr] gap-4">
      <ScrollArea className="h-full min-h-0 rounded-md border p-1">
        <TraceActionList trace={trace} selected={selected?.id} onSelect={setSelectedId} />
      </ScrollArea>
      <div className="flex min-h-0 min-w-0 flex-col gap-3">
        <ActionHeader trace={trace} action={selected} moment={moment} onMoment={setMoment} />
        <Screenshot trace={trace} action={selected} moment={moment} />
        <TraceTabs trace={trace} action={selected} onSelect={setSelectedId} />
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
  if (!trace) return <LoadingPanel />;
  return <TraceDetails trace={trace} />;
}
