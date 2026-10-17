import { CircleCheck, CircleX, Maximize2, Radio, X, type LucideIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
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
import { Ripple } from "../app/motion";
import { useLiveStore, type LiveCapture } from "../store/live";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { FrameOverlay } from "./frame-overlay";
import { ListRow } from "./list-row";
import { LogView } from "./log-view";
import { StatusIcon } from "./status-icon";

const RIPPLE_MS = 600;
const RIPPLE_START_PX = 12;
const RIPPLE_GROWTH_PX = 36;
const RIPPLE_DESIGN_WIDTH = 1920;
const RIPPLE_STROKE = 1 / 640;
const CURSOR_SIZE = 1 / 220;

const CAPTURE_STATUS: Record<
  LiveCapture["status"],
  {
    icon: LucideIcon;
    badge: string;
    word: string;
    variant: "default" | "secondary" | "destructive";
  }
> = {
  live: { icon: Radio, badge: "Live", word: "Recording", variant: "default" },
  done: { icon: CircleCheck, badge: "Done", word: "Done", variant: "secondary" },
  error: { icon: CircleX, badge: "Failed", word: "Failed", variant: "destructive" },
};

function StatusBadge({ status }: { status: LiveCapture["status"] }) {
  const { icon: Icon, badge, variant } = CAPTURE_STATUS[status];
  return (
    <Badge variant={variant}>
      <Icon data-icon="inline-start" />
      {badge}
    </Badge>
  );
}

function ClickRipple({ click, width }: { click: LiveCapture["clicks"][number]; width: number }) {
  const [age] = useState(() => Date.now() - click.at);
  if (age >= RIPPLE_MS) return null;
  const scale = width / RIPPLE_DESIGN_WIDTH;
  const start = age / RIPPLE_MS;
  return (
    <Ripple
      cx={click.x}
      cy={click.y}
      from={(RIPPLE_START_PX + start * RIPPLE_GROWTH_PX) * scale}
      to={(RIPPLE_START_PX + RIPPLE_GROWTH_PX) * scale}
      opacity={1 - start}
      strokeWidth={width * RIPPLE_STROKE}
      seconds={(RIPPLE_MS - age) / 1000}
      className="fill-none stroke-primary"
    />
  );
}

function LiveView({ capture, className }: { capture: LiveCapture; className?: string }) {
  const { width, height } = capture.viewport;
  const current = capture.steps.find((s) => s.status === "running");
  const target = capture.target;
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
      <FrameOverlay
        width={width}
        height={height}
        box={
          target
            ? { x: target.rect.x, y: target.rect.y, width: target.rect.w, height: target.rect.h }
            : null
        }
        point={capture.cursor}
        pointSize={CURSOR_SIZE}
      >
        {capture.clicks.map((c) => (
          <ClickRipple key={c.at} click={c} width={width} />
        ))}
      </FrameOverlay>
      <div className="absolute inset-x-2 bottom-2 flex items-center gap-1.5">
        <StatusBadge status={capture.status} />
        {current ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {current.label}
          </Badge>
        ) : null}
        {target?.label && target.label !== current?.label ? (
          <Badge variant="outline" className="bg-background/80 truncate">
            {target.label}
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

function PipWindow({ capture }: { capture: LiveCapture }) {
  const order = useLiveStore((s) => s.order);
  const titles = useLiveStore(useShallow((s) => s.order.map((id) => s.captures[id].title)));
  const { focus, setPipOpen, setDialogOpen } = useLiveStore.getState();
  return (
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
          {order.map((id, i) => (
            <Button
              key={id}
              size="xs"
              variant={id === capture.captureId ? "secondary" : "ghost"}
              onClick={() => focus(id)}
            >
              {titles[i]}
            </Button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function LiveDialog({ capture }: { capture: LiveCapture }) {
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
              {CAPTURE_STATUS[capture.status].word}
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

export function LivePip() {
  const capture = useLiveStore((s) => (s.focusId ? s.captures[s.focusId] : undefined));
  const pipOpen = useLiveStore((s) => s.pipOpen);
  if (!capture) return null;
  return (
    <>
      {pipOpen ? <PipWindow capture={capture} /> : null}
      <LiveDialog capture={capture} />
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
