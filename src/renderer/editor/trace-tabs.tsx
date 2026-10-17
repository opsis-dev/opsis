import { CircleAlert } from "lucide-react";
import type { ReactNode } from "react";
import type { TraceAction, TraceConsole, TraceModel } from "@/shared/trace";
import { formatBytes, formatDuration } from "@/shared/format";
import { Badge } from "@/renderer/components/ui/badge";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/renderer/components/ui/table";
import { cn } from "@/renderer/lib/utils";
import { MONO_TEXT } from "./log-view";

const LEVEL_VARIANTS: Record<TraceConsole["level"], "destructive" | "secondary" | "outline"> = {
  error: "destructive",
  warning: "secondary",
  log: "outline",
  info: "outline",
  debug: "outline",
};

const within = (time: number, action: TraceAction | undefined) =>
  !!action && time >= action.start && time <= action.end;

export const traceClock = (trace: TraceModel) => (t: number) =>
  `+${formatDuration(t - trace.start, "precise")}`;

function TabScroll({ children, className }: { children: ReactNode; className?: string }) {
  return <ScrollArea className={cn("h-48 rounded-md", className)}>{children}</ScrollArea>;
}

export function CallTab({ action }: { action: TraceAction | undefined }) {
  return (
    <TabScroll className="bg-muted/40">
      <div className={cn(MONO_TEXT, "flex flex-col gap-1 p-2 leading-relaxed")}>
        {action?.error ? (
          <div className="text-destructive whitespace-pre-wrap">{action.error}</div>
        ) : null}
        {action?.target ? <div>{action.target}</div> : null}
        {(action?.logs ?? []).map((l, i) => (
          <div key={i} className="text-muted-foreground whitespace-pre-wrap">
            {l}
          </div>
        ))}
      </div>
    </TabScroll>
  );
}

export function ConsoleTab({
  trace,
  action,
}: {
  trace: TraceModel;
  action: TraceAction | undefined;
}) {
  const at = traceClock(trace);
  return (
    <TabScroll className="border">
      <ol className={cn(MONO_TEXT, "flex flex-col")}>
        {trace.console.map((c, i) => (
          <li
            key={i}
            className={cn(
              "flex items-start gap-2 border-b px-2 py-1",
              within(c.time, action) && "bg-muted",
            )}
          >
            <span className="text-muted-foreground w-14 shrink-0 tabular-nums">{at(c.time)}</span>
            <Badge variant={LEVEL_VARIANTS[c.level]}>{c.level}</Badge>
            <span className="min-w-0 flex-1 break-all whitespace-pre-wrap">{c.text}</span>
          </li>
        ))}
      </ol>
    </TabScroll>
  );
}

export function NetworkTab({
  trace,
  action,
}: {
  trace: TraceModel;
  action: TraceAction | undefined;
}) {
  const at = traceClock(trace);
  return (
    <TabScroll className="border">
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
            <TableRow key={i} data-state={within(r.start, action) ? "selected" : undefined}>
              <TableCell className="tabular-nums">{at(r.start)}</TableCell>
              <TableCell>{r.method}</TableCell>
              <TableCell>
                {r.failure ? (
                  <Badge variant="destructive">{r.failure}</Badge>
                ) : (
                  <Badge variant={r.status >= 400 ? "destructive" : "outline"}>{r.status}</Badge>
                )}
              </TableCell>
              <TableCell className="max-w-96 truncate font-mono" title={r.url}>
                {r.url}
              </TableCell>
              <TableCell className="text-muted-foreground">
                {r.resourceType || r.mimeType.split(";")[0]}
              </TableCell>
              <TableCell className="text-right tabular-nums">{formatBytes(r.size)}</TableCell>
              <TableCell className="text-right tabular-nums">
                {formatDuration(r.duration, "precise")}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TabScroll>
  );
}

export function ErrorsTab({
  trace,
  onSelect,
}: {
  trace: TraceModel;
  onSelect: (actionId: string) => void;
}) {
  const at = traceClock(trace);
  return (
    <TabScroll className="border">
      {trace.errors.length === 0 ? (
        <p className="text-muted-foreground p-2 text-xs">No errors.</p>
      ) : (
        <ol className={cn(MONO_TEXT, "flex flex-col")}>
          {trace.errors.map((e, i) => (
            <li key={i} className="flex items-start gap-2 border-b px-2 py-1">
              <CircleAlert className="text-destructive size-3.5 shrink-0" />
              <span className="text-muted-foreground w-14 shrink-0 tabular-nums">{at(e.time)}</span>
              <button
                type="button"
                className="min-w-0 flex-1 text-left break-all whitespace-pre-wrap"
                onClick={() => e.actionId && onSelect(e.actionId)}
              >
                {e.stack ?? e.message}
              </button>
            </li>
          ))}
        </ol>
      )}
    </TabScroll>
  );
}
