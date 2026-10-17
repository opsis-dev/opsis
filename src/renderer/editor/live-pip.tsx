import { Maximize2, Radio, X } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { Button } from "@/renderer/components/ui/button";
import { useLiveStore, type LiveCapture } from "../store/live";
import { LiveDialog } from "./live-dialog";
import { LiveView } from "./live-view";

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
