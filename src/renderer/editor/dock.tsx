import { ChevronDown, ChevronUp, SquareArrowOutUpRight } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { useWorkflowStore } from "../store/workflow";
import { MixerPanel } from "./mixer/mixer-panel";
import {
  DOCK_PANELS,
  DOCK_TITLES,
  focusPanel,
  openPanel,
  showPanel,
  togglePanel,
  usePanels,
  type DockPanel,
} from "./panels";
import { PreviewDock } from "./preview-dock";
import { RunPanel } from "./run-panel";
import { TimelinePanel } from "./timeline/timeline-panel";
import { shortcutLabel } from "../app/app-bar";

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

export function popOut(panel: DockPanel): void {
  const dir = useWorkflowStore.getState().projectDir;
  if (!dir) return;
  const { setPopped, popped, active, setActive } = usePanels.getState();
  setPopped({ [panel]: true });
  if (active === panel) {
    const next = DOCK_PANELS.find((p) => p !== panel && !popped[p]);
    if (next) setActive(next);
  }
  void window.demoflow.openPanelWindow(panel, dir);
}

function syncPopped(list: DockPanel[]) {
  const { setPopped, active, setActive } = usePanels.getState();
  setPopped(Object.fromEntries(DOCK_PANELS.map((p) => [p, list.includes(p)])));
  if (list.includes(active)) {
    const next = DOCK_PANELS.find((p) => !list.includes(p));
    if (next) setActive(next);
  }
}

export function BottomDock() {
  const open = usePanels((s) => s.open.bottom);
  const active = usePanels((s) => s.active);
  const popped = usePanels((s) => s.popped);
  const docked = DOCK_PANELS.filter((p) => !popped[p]);
  const current = docked.includes(active) ? active : (docked[0] ?? active);

  useEffect(() => {
    void window.demoflow.listPanelWindows().then(syncPopped);
    return window.demoflow.onPanelWindows(syncPopped);
  }, []);

  return (
    <Tabs
      value={current}
      onValueChange={(v) => showPanel(v as DockPanel)}
      className="flex h-full flex-col gap-0"
    >
      <div className="flex h-9 shrink-0 items-center gap-1 border-b pr-1">
        <TabsList variant="line" className="h-9 flex-1 justify-start px-2">
          {DOCK_PANELS.map((panel, i) =>
            popped[panel] ? (
              <Tooltip key={panel}>
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
            ) : (
              <Tooltip key={panel}>
                <TooltipTrigger
                  render={
                    <TabsTrigger
                      value={panel}
                      className="flex-none"
                      onClick={() => openPanel("bottom")}
                    />
                  }
                >
                  {DOCK_TITLES[panel]}
                </TooltipTrigger>
                <TooltipContent>
                  {DOCK_TITLES[panel]} <Kbd>{shortcutLabel(`CmdOrCtrl+${i + 1}`)}</Kbd>
                </TooltipContent>
              </Tooltip>
            ),
          )}
        </TabsList>
        {docked.includes(current) ? (
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-xs"
                  aria-label={`Open ${DOCK_TITLES[current].toLowerCase()} in a new window`}
                  onClick={() => popOut(current)}
                />
              }
            >
              <SquareArrowOutUpRight />
            </TooltipTrigger>
            <TooltipContent>
              Open {DOCK_TITLES[current].toLowerCase()} in a new window
            </TooltipContent>
          </Tooltip>
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
      {docked.length === 0 ? (
        <div className="text-muted-foreground grid flex-1 place-items-center text-sm">
          Every tab is open in its own window.
        </div>
      ) : null}
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
