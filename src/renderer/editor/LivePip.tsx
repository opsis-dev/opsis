import { CircleCheck, CircleX, Maximize2, Radio, X } from "lucide-react";
import { useEffect, useState } from "react";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/renderer/components/ui/dialog";
import { ScrollArea } from "@/renderer/components/ui/scroll-area";
import { Spinner } from "@/renderer/components/ui/spinner";
import { cn } from "@/renderer/lib/utils";
import { useLiveStore, type LiveCapture } from "../store/live";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";

function useNow(active: boolean) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return;
    const t = setInterval(() => setNow(Date.now()), 50);
    return () => clearInterval(t);
  }, [active]);
  return now;
}

function LiveView({ capture, className }: { capture: LiveCapture; className?: string }) {
  const now = useNow(capture.status === "live");
  const { width, height } = capture.viewport;
  const current = capture.steps.find((s) => s.status === "running");
  return (
    <div
      className={cn("relative overflow-hidden rounded-md bg-black", className)}
      style={{ aspectRatio: `${width} / ${height}` }}
    >
      {capture.frameUrl ? (
        <img
          src={capture.frameUrl}
          alt={capture.title}
          className="absolute inset-0 size-full object-contain"
        />
      ) : (
        <div className="text-muted-foreground absolute inset-0 grid place-items-center text-xs">
          <Spinner />
        </div>
      )}
      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="pointer-events-none absolute inset-0 size-full"
      >
        {capture.target ? (
          <rect
            x={capture.target.rect.x}
            y={capture.target.rect.y}
            width={capture.target.rect.w}
            height={capture.target.rect.h}
            rx={8}
            className="fill-primary/10 stroke-primary"
            strokeWidth={width / 480}
          />
        ) : null}
        {capture.clicks
          .filter((c) => now - c.at < 600)
          .map((c) => {
            const p = (now - c.at) / 600;
            return (
              <circle
                key={c.at}
                cx={c.x}
                cy={c.y}
                r={(12 + p * 36) * (width / 1920)}
                className="fill-none stroke-primary"
                strokeWidth={width / 640}
                opacity={1 - p}
              />
            );
          })}
        {capture.cursor ? (
          <circle
            cx={capture.cursor.x}
            cy={capture.cursor.y}
            r={width / 220}
            className="fill-primary stroke-background"
            strokeWidth={width / 960}
          />
        ) : null}
      </svg>
      <div className="absolute inset-x-2 bottom-2 flex items-center gap-1.5">
        {capture.status === "live" ? (
          <Badge>
            <Radio data-icon="inline-start" />
            Live
          </Badge>
        ) : capture.status === "done" ? (
          <Badge variant="secondary">
            <CircleCheck data-icon="inline-start" />
            Done
          </Badge>
        ) : (
          <Badge variant="destructive">
            <CircleX data-icon="inline-start" />
            Failed
          </Badge>
        )}
        {current ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {current.label}
          </Badge>
        ) : null}
        {capture.target?.label && capture.target.label !== current?.label ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {capture.target.label}
          </Badge>
        ) : null}
      </div>
    </div>
  );
}

function StepList({ capture }: { capture: LiveCapture }) {
  const setSelected = useUiStore((s) => s.setSelected);
  return (
    <ol className="flex flex-col gap-1">
      {capture.steps.map((step, i) => (
        <li key={step.nodeId}>
          <Button
            variant="ghost"
            size="sm"
            className="h-auto w-full justify-start gap-2 py-1.5 text-left"
            onClick={() => setSelected([step.nodeId])}
          >
            <span className="text-muted-foreground w-5 shrink-0 text-right tabular-nums">
              {i + 1}
            </span>
            {step.status === "running" ? (
              <Spinner />
            ) : step.status === "done" ? (
              <CircleCheck className="text-primary" />
            ) : step.status === "error" ? (
              <CircleX className="text-destructive" />
            ) : (
              <span className="size-4" />
            )}
            <span className="flex flex-col">
              <span className="truncate">{step.label}</span>
              {step.message ? (
                <span className="text-destructive text-xs whitespace-normal">{step.message}</span>
              ) : null}
            </span>
          </Button>
        </li>
      ))}
    </ol>
  );
}

export function LivePip() {
  const { captures, order, pipOpen, focusId, dialogOpen, setPipOpen, focus, setDialogOpen } =
    useLiveStore();
  const logs = useRunStore((s) => s.logs);
  const capture = focusId ? captures[focusId] : undefined;
  if (!capture) return null;
  const nodeLogs = logs.filter((l) => l.nodeId === capture.nodeId);

  return (
    <>
      {pipOpen ? (
        <div className="bg-card absolute right-4 bottom-4 z-10 flex w-96 flex-col gap-2 rounded-xl border p-2 shadow-xl">
          <div className="flex items-center gap-1">
            <span className="flex-1 truncate px-1 text-sm font-medium">{capture.title}</span>
            {order.length > 1 ? (
              <span className="text-muted-foreground text-xs tabular-nums">
                {order.indexOf(capture.captureId) + 1}/{order.length}
              </span>
            ) : null}
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Open large view"
              onClick={() => setDialogOpen(true)}
            >
              <Maximize2 />
            </Button>
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Close live view"
              onClick={() => setPipOpen(false)}
            >
              <X />
            </Button>
          </div>
          <button type="button" className="cursor-zoom-in" onClick={() => setDialogOpen(true)}>
            <LiveView capture={capture} />
          </button>
          {order.length > 1 ? (
            <div className="flex gap-1 overflow-x-auto">
              {order.map((id) => (
                <Button
                  key={id}
                  size="xs"
                  variant={id === capture.captureId ? "secondary" : "ghost"}
                  onClick={() => focus(id)}
                >
                  {captures[id].title}
                </Button>
              ))}
            </div>
          ) : null}
        </div>
      ) : null}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="grid max-h-[92vh] w-[92vw] max-w-[92vw] grid-cols-[1fr_320px] gap-4 sm:max-w-[92vw]">
          <div className="flex min-w-0 flex-col gap-3">
            <DialogHeader>
              <DialogTitle>{capture.title}</DialogTitle>
              <DialogDescription>
                {capture.viewport.width}×{capture.viewport.height} ·{" "}
                {capture.status === "live"
                  ? "Recording"
                  : capture.status === "done"
                    ? "Done"
                    : "Failed"}
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
              <pre className="p-2 font-mono text-[11px] leading-relaxed whitespace-pre-wrap">
                {nodeLogs.map((l, i) => (
                  <div key={i} className={cn(l.level === "error" && "text-destructive")}>
                    {l.message}
                  </div>
                ))}
              </pre>
            </ScrollArea>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}

export function LiveToggle() {
  const order = useLiveStore((s) => s.order);
  const pipOpen = useLiveStore((s) => s.pipOpen);
  const setPipOpen = useLiveStore((s) => s.setPipOpen);
  if (!order.length || pipOpen) return null;
  return (
    <Button variant="ghost" size="sm" onClick={() => setPipOpen(true)}>
      <Radio data-icon="inline-start" />
      Live view
    </Button>
  );
}
