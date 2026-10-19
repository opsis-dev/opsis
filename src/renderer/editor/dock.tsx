import { ChevronDown, ChevronUp, SquareArrowOutUpRight } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/renderer/components/ui/tabs";
import { whenOneOf } from "@/renderer/lib/one-of";
import { DOCK_PANELS, DOCK_TITLES, type DockPanelName } from "@/shared/ipc";
import { commandShortcut } from "../app/menu-shortcuts";
import { MixerPanel } from "./mixer/mixer-panel";
import { focusPanel, openPanel, showPanel, togglePanel, usePanels } from "./panels";
import { PreviewDock } from "./preview-dock";
import { RunPanel } from "./run-panel";
import { TimelinePanel } from "./timeline/timeline-panel";
import { TooltipButton, WithTooltip } from "./with-tooltip";

export const DOCK_TAB_BAR_PX = 36;

export function DockContent({ panel }: { panel: DockPanelName }) {
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

function markPopped(popped: readonly DockPanelName[]): void {
  const { setPopped, active, setActive } = usePanels.getState();
  setPopped(popped);
  if (!popped.includes(active)) return;
  const next = DOCK_PANELS.find((p) => !popped.includes(p));
  if (next) setActive(next);
}

function popOut(panel: DockPanelName): void {
  markPopped([...usePanels.getState().popped, panel]);
  void window.opsis.openPanelWindow(panel);
}

function DockTab({ panel, popped }: { panel: DockPanelName; popped: boolean }) {
  const title = DOCK_TITLES[panel];
  return (
    <WithTooltip
      trigger={
        popped ? (
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground flex h-full items-center gap-1 px-2 text-sm"
            onClick={() => focusPanel(panel)}
          />
        ) : (
          <TabsTrigger value={panel} className="flex-none" onClick={() => openPanel("bottom")} />
        )
      }
      tip={
        popped ? (
          "Open in its own window. Close that window to bring it back here."
        ) : (
          <>
            {title} <Kbd>{commandShortcut(`view:focus-${panel}`)}</Kbd>
          </>
        )
      }
    >
      {title}
      {popped ? <SquareArrowOutUpRight className="size-3" /> : null}
    </WithTooltip>
  );
}

function DockTabStrip({
  popped,
  current,
}: {
  popped: readonly DockPanelName[];
  current: DockPanelName | null;
}) {
  const open = usePanels((s) => s.open.bottom);
  return (
    <div
      className="flex shrink-0 items-center gap-1 border-b pr-1"
      style={{ height: DOCK_TAB_BAR_PX }}
    >
      <TabsList
        variant="line"
        className="flex-1 justify-start px-2"
        style={{ height: DOCK_TAB_BAR_PX }}
      >
        {DOCK_PANELS.map((panel) => (
          <DockTab key={panel} panel={panel} popped={popped.includes(panel)} />
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
  );
}

export function BottomDock() {
  const active = usePanels((s) => s.active);
  const popped = usePanels((s) => s.popped);
  const docked = DOCK_PANELS.filter((p) => !popped.includes(p));
  const current: DockPanelName | null = docked.includes(active) ? active : (docked[0] ?? null);

  useEffect(() => {
    void window.opsis.listPanelWindows().then(markPopped);
    return window.opsis.onPanelWindows(markPopped);
  }, []);

  return (
    <Tabs
      value={current ?? active}
      onValueChange={whenOneOf(DOCK_PANELS, showPanel)}
      className="flex h-full flex-col gap-0"
    >
      <DockTabStrip popped={popped} current={current} />
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
