import { findById } from "@opsis/video/common/lists";
import { Maximize2, Radio, X } from "lucide-react";
import { useEffect } from "react";
import { useShallow } from "zustand/react/shallow";
import { on, watchLive } from "@/app/tauri";
import { Button } from "@/components/ui/button";
import { commands, events } from "@/generated/bindings";
import { nodeName } from "@/store/catalog";
import { useLiveStore, type LiveCapture } from "../store/live";
import { useWorkflowStore } from "../store/workflow";
import { LiveView } from "./live-view";

function openLiveWindow() {
  return void commands.openLiveWindow();
}

function useCaptureTitle(capture: LiveCapture): string {
  const node = useWorkflowStore((s) => findById(s.workflow?.nodes, capture.nodeId));
  return capture.title || (node ? nodeName(node) : "Live recording");
}

function PipWindow({ capture }: { capture: LiveCapture }) {
  const captures = useLiveStore(useShallow((s) => s.order.map((id) => s.captures[id])));
  const focus = useLiveStore((s) => s.focus);
  const setPipOpen = useLiveStore((s) => s.setPipOpen);
  const title = useCaptureTitle(capture);
  return (
    <div className="bg-card absolute right-4 bottom-4 z-10 flex w-96 flex-col gap-2 rounded-xl border p-2 shadow-xl">
      <div className="flex items-center gap-1">
        <span className="flex-1 truncate px-1 text-sm font-medium">{title}</span>
        {captures.length > 1 ? (
          <span className="text-muted-foreground text-xs tabular-nums">
            {captures.findIndex((c) => c.captureId === capture.captureId) + 1}/{captures.length}
          </span>
        ) : null}
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label="Open in its own window"
          onClick={openLiveWindow}
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
      <button type="button" className="cursor-zoom-in" onClick={openLiveWindow}>
        <LiveView capture={capture} />
      </button>
      {captures.length > 1 ? (
        <div className="flex gap-1 overflow-x-auto">
          {captures.map((c) => (
            <Button
              key={c.captureId}
              size="xs"
              variant={c.captureId === capture.captureId ? "secondary" : "ghost"}
              onClick={() => focus(c.captureId)}
            >
              {c.title || c.nodeId}
            </Button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function useLiveWindowWatching(): boolean {
  const watching = useLiveStore((s) => s.windowWatching);
  useEffect(() => {
    const { setWindowWatching } = useLiveStore.getState();
    void commands.liveWindowWatching().then(setWindowWatching);
    return on(events.liveWindowWatching, setWindowWatching);
  }, []);
  return watching;
}

export function LivePip() {
  const windowWatching = useLiveWindowWatching();
  useEffect(() => {
    if (windowWatching) return;
    const live = useLiveStore.getState();
    live.reset();
    return watchLive(live.apply);
  }, [windowWatching]);
  const capture = useLiveStore((s) => (s.focusId ? s.captures[s.focusId] : undefined));
  const pipOpen = useLiveStore((s) => s.pipOpen);
  if (!capture || windowWatching || !pipOpen) return null;
  return <PipWindow capture={capture} />;
}

export function LiveToggle() {
  const order = useLiveStore((s) => s.order);
  const pipOpen = useLiveStore((s) => s.pipOpen);
  const windowWatching = useLiveStore((s) => s.windowWatching);
  const setPipOpen = useLiveStore((s) => s.setPipOpen);
  if (!order.length || pipOpen || windowWatching) return null;
  return (
    <Button variant="ghost" size="sm" onClick={() => setPipOpen(true)}>
      <Radio data-icon="inline-start" />
      Live view
    </Button>
  );
}
