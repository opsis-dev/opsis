import { ChevronDown, ChevronUp, SquareArrowOutUpRight } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { commandShortcut } from "../app/app-bar";
import { useWorkflowStore } from "../store/workflow";
import { MixerPanel } from "./mixer/mixer-panel";
import {
  DOCK_PANELS,
  DOCK_TITLES,
  focusPanel,
  isDockPanel,
  openPanel,
  showPanel,
  togglePanel,
  usePanels,
  type DockPanel,
} from "./panels";
import { PreviewDock } from "./preview-dock";
import { RunPanel } from "./run-panel";
import { TimelinePanel } from "./timeline/timeline-panel";
import { TooltipButton } from "./tooltip-button";

export function DockContent({ panel }: { panel: DockPanel }) {
  switch (panel) {
    case "preview":
      return <PreviewDock />;
    case "timeline":
      return <TimelinePanel />;
    case "mixer":
      return <MixerPanel />;
    case "run":
      return <RunPanel />;
  }
}

function markPopped(popped: Record<DockPanel, boolean>): void {
  const { setPopped, active, setActive } = usePanels.getState();
  setPopped(popped);
  if (!popped[active]) return;
  const next = DOCK_PANELS.find((p) => !popped[p]);
  if (next) setActive(next);
}

function popOut(panel: DockPanel): void {
  const dir = useWorkflowStore.getState().projectDir;
  if (!dir) return;
  markPopped({ ...usePanels.getState().popped, [panel]: true });
  void window.demoflow.openPanelWindow(panel, dir);
}

const syncPopped = (list: DockPanel[]) =>
  markPopped({
    preview: list.includes("preview"),
    timeline: list.includes("timeline"),
    mixer: list.includes("mixer"),
    run: list.includes("run"),
  });

function DockTab({ panel, popped }: { panel: DockPanel; popped: boolean }) {
  if (popped)
    return (
      <Tooltip>
        <TooltipTrigger
          render={
            <button
              type="button"
              className="text-muted-foreground hover:text-foreground flex h-full items-center gap-1 px-2 text-sm"
              onClick={() => focusPanel(panel)}
            />
          }
        >
          {DOCK_TITLES[panel]}
          <SquareArrowOutUpRight className="size-3" />
        </TooltipTrigger>
        <TooltipContent>
          Open in its own window. Close that window to bring it back here.
        </TooltipContent>
      </Tooltip>
    );
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <TabsTrigger value={panel} className="flex-none" onClick={() => openPanel("bottom")} />
        }
      >
        {DOCK_TITLES[panel]}
      </TooltipTrigger>
      <TooltipContent>
        {DOCK_TITLES[panel]} <Kbd>{commandShortcut(`view:focus-${panel}`)}</Kbd>
      </TooltipContent>
    </Tooltip>
  );
}

export function BottomDock() {
  const open = usePanels((s) => s.open.bottom);
  const active = usePanels((s) => s.active);
  const popped = usePanels((s) => s.popped);
  const docked = DOCK_PANELS.filter((p) => !popped[p]);
  const current: DockPanel | null = docked.includes(active) ? active : (docked[0] ?? null);

  useEffect(() => {
    void window.demoflow.listPanelWindows().then(syncPopped);
    return window.demoflow.onPanelWindows(syncPopped);
  }, []);

  return (
    <Tabs
      value={current ?? active}
      onValueChange={(v) => isDockPanel(v) && showPanel(v)}
      className="flex h-full flex-col gap-0"
    >
      <div className="flex h-9 shrink-0 items-center gap-1 border-b pr-1">
        <TabsList variant="line" className="h-9 flex-1 justify-start px-2">
          {DOCK_PANELS.map((panel) => (
            <DockTab key={panel} panel={panel} popped={popped[panel]} />
          ))}
        </TabsList>
        {current ? (
          <TooltipButton
            label={`Open ${DOCK_TITLES[current].toLowerCase()} in a new window`}
            icon={SquareArrowOutUpRight}
            size="icon-xs"
            onClick={() => popOut(current)}
          />
        ) : null}
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label={open ? "Hide the bottom panel" : "Show the bottom panel"}
          onClick={() => togglePanel("bottom")}
        >
          {open ? <ChevronDown /> : <ChevronUp />}
        </Button>
      </div>
      {current ? null : (
        <div className="text-muted-foreground grid flex-1 place-items-center text-sm">
          Every tab is open in its own window.
        </div>
      )}
      {docked.map((panel) => (
        <TabsContent
          key={panel}
          value={panel}
          keepMounted={panel === "preview"}
          id={`panel-${panel}`}
          tabIndex={-1}
          className="min-h-0 flex-1 outline-none data-[hidden]:hidden"
        >
          <DockContent panel={panel} />
        </TabsContent>
      ))}
    </Tabs>
  );
}
