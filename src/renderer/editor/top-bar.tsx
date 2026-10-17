import { useNavigate } from "@tanstack/react-router";
import {
  ArrowLeft,
  FolderOpen,
  LayoutGrid,
  Play,
  Plus,
  Redo2,
  Settings,
  Undo2,
} from "lucide-react";
import { useStore } from "zustand";
import { Badge } from "@/renderer/components/ui/badge";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Separator } from "@/renderer/components/ui/separator";
import { Spinner } from "@/renderer/components/ui/spinner";
import { Toggle } from "@/renderer/components/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { AppBar } from "../app/app-bar";
import { commandLabel, commandShortcut } from "../app/menu-shortcuts";
import { useRunStore } from "../store/run";
import { useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";
import { tidyLayout } from "./layout";
import { LiveToggle } from "./live-pip";
import { PANEL_SIDES, PANEL_TOGGLES, togglePanel, usePanels } from "./panels";
import { revealOutputs } from "./reveal";
import { startRun } from "./run";
import { RunningLabel, StopRunButton } from "./run-controls";
import { TooltipButton } from "./tooltip-button";

const PANEL_BUTTONS = PANEL_SIDES.map((side) => ({
  side,
  icon: PANEL_TOGGLES[side].icon,
  label: commandLabel(PANEL_TOGGLES[side].command).toLowerCase(),
  keys: commandShortcut(PANEL_TOGGLES[side].command),
}));

function PanelToggles() {
  const open = usePanels((s) => s.open);
  return (
    <div className="flex items-center gap-0.5">
      {PANEL_BUTTONS.map(({ side, icon: Icon, label, keys }) => (
        <Tooltip key={side}>
          <TooltipTrigger
            render={
              <Toggle
                size="sm"
                aria-label={`${open[side] ? "Hide" : "Show"} ${label}`}
                pressed={open[side]}
                onPressedChange={() => togglePanel(side)}
              />
            }
          >
            <Icon />
          </TooltipTrigger>
          <TooltipContent>
            {open[side] ? "Hide" : "Show"} {label} <Kbd>{keys}</Kbd>
          </TooltipContent>
        </Tooltip>
      ))}
    </div>
  );
}

function HistoryButtons() {
  const canUndo = useStore(useWorkflowStore.temporal, (s) => s.pastStates.length > 0);
  const canRedo = useStore(useWorkflowStore.temporal, (s) => s.futureStates.length > 0);
  const { undo, redo } = useWorkflowStore.getState();
  return (
    <>
      <TooltipButton
        label="Undo"
        icon={Undo2}
        shortcut={commandShortcut("edit:undo")}
        disabled={!canUndo}
        onClick={() => undo()}
      />
      <TooltipButton
        label="Redo"
        icon={Redo2}
        shortcut={commandShortcut("edit:redo")}
        disabled={!canRedo}
        onClick={() => redo()}
      />
    </>
  );
}

function RunControls() {
  const running = useRunStore((s) => s.running);
  if (!running)
    return (
      <Button size="sm" onClick={() => void startRun()}>
        <Play data-icon="inline-start" />
        Run
      </Button>
    );
  return (
    <>
      <span className="flex items-center gap-1.5 text-sm">
        <Spinner className="text-muted-foreground" />
        <RunningLabel />
      </span>
      <StopRunButton size="sm" />
    </>
  );
}

export function TopBar({ projectDir }: { projectDir: string }) {
  const navigate = useNavigate();
  const name = useWorkflowStore((s) => s.workflow?.name);
  const dirty = useWorkflowStore((s) => s.revision !== s.savedRevision);

  return (
    <AppBar>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="All projects"
        onClick={() => void navigate({ to: "/" })}
      >
        <ArrowLeft />
      </Button>
      <span className="truncate font-semibold">{name}</span>
      <Badge variant="outline" className="font-normal">
        {dirty ? "Unsaved" : "Saved"}
      </Badge>
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <HistoryButtons />
      <TooltipButton
        label="Tidy up layout"
        icon={LayoutGrid}
        shortcut={commandShortcut("edit:layout")}
        onClick={() => void tidyLayout()}
      />
      <Button variant="ghost" size="sm" onClick={() => useUiStore.getState().setPaletteOpen(true)}>
        <Plus data-icon="inline-start" />
        Add node <Kbd>{commandShortcut("edit:add-node")}</Kbd>
      </Button>
      <div className="flex-1" />
      <PanelToggles />
      <Separator orientation="vertical" className="mx-1 h-5 data-vertical:self-center" />
      <LiveToggle />
      <Button variant="ghost" size="sm" onClick={() => void revealOutputs(projectDir)}>
        <FolderOpen data-icon="inline-start" />
        Outputs
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Settings"
        onClick={() => void window.demoflow.openSettings()}
      >
        <Settings />
      </Button>
      <RunControls />
    </AppBar>
  );
}
